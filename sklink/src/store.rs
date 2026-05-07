use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::AppError;
use crate::git_source;
use crate::install;
use crate::path_utils;
use crate::skills;

pub fn default_store_dir(cwd: &Path) -> Result<PathBuf, AppError> {
    path_utils::resolve_path("~/.config/sklink/skills", cwd)
}

pub fn stage_skill_to_store(
    store_dir: &Path,
    skill_name: &str,
    source_dir: &Path,
    force: bool,
) -> Result<PathBuf, AppError> {
    std::fs::create_dir_all(store_dir).map_err(|e| AppError::CreateDir {
        dir: store_dir.to_path_buf(),
        source: e,
    })?;

    let dest_dir = store_dir.join(skill_name);
    if dest_dir.exists() {
        if !force {
            return Err(AppError::StoreSkillAlreadyExists {
                skill: skill_name.to_string(),
                path: dest_dir,
            });
        }

        let backup_dir = store_backup_dir(store_dir, skill_name)?;
        if let Some(parent) = backup_dir.parent() {
            std::fs::create_dir_all(parent).map_err(|e| AppError::CreateDir {
                dir: parent.to_path_buf(),
                source: e,
            })?;
        }
        std::fs::rename(&dest_dir, &backup_dir).map_err(|e| AppError::StoreSkillBackup {
            from: dest_dir.clone(),
            to: backup_dir,
            source: e,
        })?;
    }

    copy_dir_recursive(source_dir, &dest_dir, false)?;
    Ok(dest_dir)
}

fn store_backup_dir(store_dir: &Path, skill_name: &str) -> Result<PathBuf, AppError> {
    let parent = store_dir.parent().unwrap_or(store_dir);
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| AppError::Io(std::io::Error::other(e)))?
        .as_secs();
    Ok(parent.join("backups").join(skill_name).join(ts.to_string()))
}

pub(crate) fn copy_dir_recursive(
    from: &Path,
    to: &Path,
    follow_symlinks: bool,
) -> Result<(), AppError> {
    std::fs::create_dir(to).map_err(|e| AppError::StoreDirCreate {
        dir: to.to_path_buf(),
        source: e,
    })?;

    let entries = std::fs::read_dir(from).map_err(|e| AppError::StoreDirRead {
        dir: from.to_path_buf(),
        source: e,
    })?;
    for entry in entries {
        let entry = entry.map_err(|e| AppError::StoreDirRead {
            dir: from.to_path_buf(),
            source: e,
        })?;

        if entry.file_name() == ".git" {
            continue;
        }

        let ty = entry.file_type().map_err(|e| AppError::StoreDirRead {
            dir: from.to_path_buf(),
            source: e,
        })?;

        let from_path = entry.path();
        let to_path = to.join(entry.file_name());

        if ty.is_dir() {
            copy_dir_recursive(&from_path, &to_path, follow_symlinks)?;
        } else if ty.is_file() {
            std::fs::copy(&from_path, &to_path).map_err(|e| AppError::StoreFileCopy {
                from: from_path,
                to: to_path,
                source: e,
            })?;
        } else if ty.is_symlink() {
            let target = std::fs::read_link(&from_path).map_err(|e| AppError::ReadLink {
                path: from_path.clone(),
                source: e,
            })?;
            if follow_symlinks {
                let resolved = if target.is_absolute() {
                    target
                } else {
                    from_path
                        .parent()
                        .expect("symlink has a parent directory")
                        .join(target)
                };
                let target_meta = std::fs::metadata(&resolved).map_err(AppError::Io)?;
                if target_meta.is_dir() {
                    copy_dir_recursive(&resolved, &to_path, follow_symlinks)?;
                } else {
                    std::fs::copy(&resolved, &to_path).map_err(|e| AppError::StoreFileCopy {
                        from: resolved,
                        to: to_path,
                        source: e,
                    })?;
                }
            } else {
                std::os::unix::fs::symlink(&target, &to_path).map_err(|e| {
                    AppError::CreateSymlink {
                        path: to_path,
                        target,
                        source: e,
                    }
                })?;
            }
        }
    }

    Ok(())
}

pub fn ensure_store_dir(store_dir: &Path) -> Result<PathBuf, AppError> {
    std::fs::create_dir_all(store_dir).map_err(|e| AppError::CreateDir {
        dir: store_dir.to_path_buf(),
        source: e,
    })?;
    std::fs::canonicalize(store_dir).map_err(AppError::Io)
}

pub fn install_into_store(
    cwd: &Path,
    store_dir: &Path,
    sources: &[String],
    force: bool,
) -> Result<Vec<skills::SkillDir>, AppError> {
    let repo_skills_dir = skills::detect_repo_skills_dir(cwd).ok();
    let mut seen = HashSet::new();
    let mut out = Vec::new();

    for raw in sources {
        if git_source::looks_like_git_url(raw) {
            let staged = git_source::stage_from_git_url(raw, store_dir, cwd, force)?;
            for skill in staged {
                if seen.insert(skill.name.clone()) {
                    out.push(skill);
                }
            }
            continue;
        }

        let (name, raw_dir) = if looks_like_path(raw) {
            let dir = path_utils::resolve_path(raw, cwd)?;
            let name = dir
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .ok_or_else(|| AppError::SkillNotFound {
                    skill: raw.clone(),
                    path: dir.clone(),
                    source: std::io::Error::other("missing directory name"),
                })?;
            (name, dir)
        } else {
            let Some(repo_skills_dir) = repo_skills_dir.as_ref() else {
                return Err(AppError::RepoSkillsDirInvalid {
                    path: cwd.join("skills"),
                    source: std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "skills dir not found",
                    ),
                });
            };
            (raw.clone(), repo_skills_dir.join(raw))
        };

        if !seen.insert(name.clone()) {
            continue;
        }

        skills::validate_skill_dir(raw, &raw_dir)?;
        let raw_dir = std::fs::canonicalize(&raw_dir).map_err(|e| AppError::SkillNotFound {
            skill: raw.clone(),
            path: raw_dir.clone(),
            source: e,
        })?;

        let dir = stage_skill_to_store(store_dir, &name, &raw_dir, force)?;
        out.push(skills::SkillDir {
            name,
            dir: std::fs::canonicalize(dir).map_err(AppError::Io)?,
        });
    }

    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

pub fn looks_like_path(raw: &str) -> bool {
    raw.contains('/') || raw.starts_with('.') || raw.starts_with('~')
}

pub fn output_from_store(
    cwd: &Path,
    store_dir: &Path,
    outputs: &[String],
    output_dir: Option<&str>,
    export: bool,
) -> Result<(), AppError> {
    let output_dir = output_dir.unwrap_or(".agent/skills");
    let output_dir = path_utils::resolve_path(output_dir, cwd)?;
    std::fs::create_dir_all(&output_dir).map_err(|e| AppError::CreateDir {
        dir: output_dir.clone(),
        source: e,
    })?;

    let mut seen = HashSet::new();
    for name in outputs {
        if !seen.insert(name.clone()) {
            continue;
        }

        let store_skill = store_dir.join(name);
        skills::validate_skill_dir(name, &store_skill)?;
        let dest = output_dir.join(name);

        if export {
            if dest.exists() {
                return Err(AppError::OutputPathExists { path: dest });
            }
            copy_dir_recursive(&store_skill, &dest, true)?;
            println!("exported {} -> {}", name, display_path(&dest));
        } else {
            match install::ensure_correct_symlink(&dest, &store_skill)? {
                install::InstallOutcome::Created => {
                    println!(
                        "created {} -> {}",
                        display_path(&dest),
                        display_path(&store_skill)
                    );
                }
                install::InstallOutcome::Skipped => {
                    println!("skipped {}", display_path(&dest));
                }
            }
        }
    }

    Ok(())
}

fn display_path(path: &Path) -> String {
    path.to_string_lossy().to_string()
}
