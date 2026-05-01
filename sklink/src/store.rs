use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::AppError;
use crate::path_utils;

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

    copy_dir_recursive(source_dir, &dest_dir)?;
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

fn copy_dir_recursive(from: &Path, to: &Path) -> Result<(), AppError> {
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

        let ty = entry.file_type().map_err(|e| AppError::StoreDirRead {
            dir: from.to_path_buf(),
            source: e,
        })?;

        let from_path = entry.path();
        let to_path = to.join(entry.file_name());

        if ty.is_dir() {
            copy_dir_recursive(&from_path, &to_path)?;
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
            std::os::unix::fs::symlink(&target, &to_path).map_err(|e| AppError::CreateSymlink {
                path: to_path,
                target,
                source: e,
            })?;
        }
    }

    Ok(())
}
