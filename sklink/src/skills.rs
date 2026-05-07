use std::path::{Path, PathBuf};

use crate::config;
use crate::error::AppError;
use crate::install;
use crate::path_utils;
use crate::store;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillDir {
    pub name: String,
    pub dir: PathBuf,
}

pub fn detect_repo_skills_dir(cwd: &Path) -> Result<PathBuf, AppError> {
    let in_repo_root = cwd.join("skills");
    let candidate = if in_repo_root.is_dir() {
        in_repo_root
    } else {
        let is_skills_dir = cwd
            .file_name()
            .and_then(|s| s.to_str())
            .map(|s| s == "skills")
            .unwrap_or(false);
        if is_skills_dir {
            cwd.to_path_buf()
        } else {
            return Err(AppError::RepoSkillsDirInvalid {
                path: in_repo_root,
                source: std::io::Error::new(std::io::ErrorKind::NotFound, "skills dir not found"),
            });
        }
    };

    std::fs::canonicalize(&candidate).map_err(|e| AppError::RepoSkillsDirInvalid {
        path: candidate,
        source: e,
    })
}

pub fn discover_skills(repo_skills_dir: &Path) -> Result<Vec<SkillDir>, AppError> {
    let entries = std::fs::read_dir(repo_skills_dir).map_err(|e| AppError::RepoSkillsDirRead {
        dir: repo_skills_dir.to_path_buf(),
        source: e,
    })?;

    let mut skills = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| AppError::RepoSkillsDirRead {
            dir: repo_skills_dir.to_path_buf(),
            source: e,
        })?;
        let file_type = entry.file_type().map_err(|e| AppError::RepoSkillsDirRead {
            dir: repo_skills_dir.to_path_buf(),
            source: e,
        })?;
        if !file_type.is_dir() {
            continue;
        }

        let name = entry.file_name().to_string_lossy().to_string();
        let raw_dir = entry.path();
        let dir = std::fs::canonicalize(&raw_dir).map_err(|e| AppError::SkillNotFound {
            skill: name.clone(),
            path: raw_dir.clone(),
            source: e,
        })?;
        skills.push(SkillDir { name, dir });
    }

    skills.sort_by(|a, b| a.name.cmp(&b.name));
    skills.dedup_by(|a, b| a.name == b.name);
    Ok(skills)
}

fn display_path(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

pub fn list_available(cwd: &Path) -> Result<(), AppError> {
    let store_dir = store::default_store_dir(cwd)?;

    let skills_dir = detect_repo_skills_dir(cwd).unwrap_or(store_dir);
    let skills = discover_skills(&skills_dir)?;
    for skill in skills {
        println!("{}", skill.name);
    }
    Ok(())
}

pub fn validate_skill_dir(skill: &str, dir: &PathBuf) -> Result<(), AppError> {
    let meta = std::fs::metadata(dir).map_err(|e| AppError::SkillNotFound {
        skill: skill.to_string(),
        path: dir.clone(),
        source: e,
    })?;
    if !meta.is_dir() {
        return Err(AppError::SkillNotFound {
            skill: skill.to_string(),
            path: dir.clone(),
            source: std::io::Error::other("not a directory"),
        });
    }
    Ok(())
}

pub fn list_installed(cwd: &Path) -> Result<(), AppError> {
    let config = config::load_default_config()?;
    let store_dir = store::default_store_dir(cwd)?;
    let store_dir = std::fs::canonicalize(&store_dir).ok();

    let mut platform_names: Vec<String> = config.platforms.keys().cloned().collect();
    platform_names.sort();

    for platform_name in platform_names {
        let Some(platform) = config.platforms.get(&platform_name) else {
            continue;
        };

        println!("{platform_name}");

        let mut target_dirs = Vec::new();
        for target in &platform.targets {
            let target_dir = path_utils::resolve_path(&target.dir, cwd)?;
            let meta = match std::fs::metadata(&target_dir) {
                Ok(m) => m,
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                    eprintln!(
                        "warning: target dir not found (skipped): platform={platform_name} dir={}",
                        display_path(&target_dir)
                    );
                    continue;
                }
                Err(err) => {
                    eprintln!(
                        "warning: failed to read target dir (skipped): platform={platform_name} dir={} err={err}",
                        display_path(&target_dir)
                    );
                    continue;
                }
            };
            if !meta.is_dir() {
                eprintln!(
                    "warning: target path is not a directory (skipped): platform={platform_name} dir={}",
                    display_path(&target_dir)
                );
                continue;
            }
            target_dirs.push(target_dir);
        }

        target_dirs.sort_by_key(|a| display_path(a));

        for (target_idx, target_dir) in target_dirs.iter().enumerate() {
            let target_prefix = if target_idx + 1 == target_dirs.len() {
                "└──"
            } else {
                "├──"
            };
            println!("{target_prefix} {}", display_path(target_dir));

            let entries = match std::fs::read_dir(target_dir) {
                Ok(e) => e,
                Err(err) => {
                    eprintln!(
                        "warning: failed to list target dir (skipped): platform={platform_name} dir={} err={err}",
                        display_path(target_dir)
                    );
                    continue;
                }
            };

            let mut rendered = Vec::new();
            for entry in entries {
                let entry = match entry {
                    Ok(e) => e,
                    Err(err) => {
                        eprintln!(
                            "warning: failed to read target entry (skipped): platform={platform_name} dir={} err={err}",
                            display_path(target_dir)
                        );
                        continue;
                    }
                };
                let name = entry.file_name().to_string_lossy().to_string();
                if name == ".DS_Store" {
                    continue;
                }

                let path = entry.path();
                let meta = match std::fs::symlink_metadata(&path) {
                    Ok(m) => m,
                    Err(err) => {
                        let line = format!("[?] {name} (error:{err})");
                        rendered.push((name, line));
                        continue;
                    }
                };

                let line = if meta.file_type().is_symlink() {
                    match std::fs::read_link(&path) {
                        Ok(raw_target) => {
                            let resolved = install::resolve_symlink_target(&path, &raw_target);
                            match std::fs::canonicalize(&resolved) {
                                Ok(resolved) => {
                                    let status = match &store_dir {
                                        Some(store_dir) => {
                                            let expected = store_dir.join(&name);
                                            let expected = std::fs::canonicalize(expected).ok();
                                            if expected.is_some_and(|e| e == resolved) {
                                                "ok".to_string()
                                            } else {
                                                "outside-store".to_string()
                                            }
                                        }
                                        None => "unknown-store".to_string(),
                                    };
                                    format!("[L] {name} -> {} ({status})", display_path(&resolved))
                                }
                                Err(err) => format!("[L] {name} (broken:{err})"),
                            }
                        }
                        Err(err) => format!("[L] {name} (broken:{err})"),
                    }
                } else if meta.is_dir() {
                    format!("[D] {name}")
                } else if meta.is_file() {
                    format!("[F] {name}")
                } else {
                    format!("[?] {name}")
                };

                rendered.push((name, line));
            }

            rendered.sort_by(|a, b| a.0.cmp(&b.0));

            for (entry_idx, (_, line)) in rendered.iter().enumerate() {
                let has_more_targets = target_idx + 1 != target_dirs.len();
                let indent = if has_more_targets { "│   " } else { "    " };
                let prefix = if entry_idx + 1 == rendered.len() {
                    "└──"
                } else {
                    "├──"
                };
                println!("{indent}{prefix} {line}");
            }
        }
    }

    Ok(())
}
