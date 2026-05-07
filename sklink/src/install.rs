use std::path::{Path, PathBuf};

use crate::config;
use crate::error::AppError;
use crate::path_utils;
use crate::skills;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallOutcome {
    Created,
    Skipped,
}

pub fn ensure_correct_symlink(
    link_path: &Path,
    link_target: &Path,
) -> Result<InstallOutcome, AppError> {
    match std::fs::symlink_metadata(link_path) {
        Ok(meta) => {
            if !meta.file_type().is_symlink() {
                return Err(AppError::LinkPathNotSymlink {
                    path: link_path.to_path_buf(),
                });
            }

            let raw_target = std::fs::read_link(link_path).map_err(|e| AppError::ReadLink {
                path: link_path.to_path_buf(),
                source: e,
            })?;

            let actual = resolve_symlink_target(link_path, &raw_target);
            let actual = std::fs::canonicalize(&actual).map_err(AppError::Io)?;
            let expected = std::fs::canonicalize(link_target).map_err(AppError::Io)?;

            if actual == expected {
                Ok(InstallOutcome::Skipped)
            } else {
                Err(AppError::LinkPathWrongTarget {
                    path: link_path.to_path_buf(),
                    actual,
                    expected,
                })
            }
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            std::os::unix::fs::symlink(link_target, link_path).map_err(|e| {
                AppError::CreateSymlink {
                    path: link_path.to_path_buf(),
                    target: link_target.to_path_buf(),
                    source: e,
                }
            })?;
            Ok(InstallOutcome::Created)
        }
        Err(err) => Err(AppError::Io(err)),
    }
}

pub fn resolve_symlink_target(link_path: &Path, raw_target: &PathBuf) -> PathBuf {
    if raw_target.is_absolute() {
        raw_target.clone()
    } else {
        let parent = link_path.parent().unwrap_or_else(|| Path::new("."));
        parent.join(raw_target)
    }
}

fn display_path(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

pub fn sync_store_to_platforms(
    cwd: &Path,
    store_dir: &Path,
    config: &config::Config,
    platform: Option<&str>,
) -> Result<(), AppError> {
    let selected_skills = skills::discover_skills(store_dir)?;

    let platform = platform.unwrap_or("all");
    let platform_names: Vec<String> = if platform == "all" {
        let mut names: Vec<String> = config.platforms.keys().cloned().collect();
        names.sort();
        names
    } else {
        if !config.platforms.contains_key(platform) {
            let mut names: Vec<String> = config.platforms.keys().cloned().collect();
            names.sort();
            return Err(AppError::PlatformNotFound {
                platform: platform.to_string(),
                available: names.join(", "),
            });
        }
        vec![platform.to_string()]
    };

    for platform_name in platform_names {
        let Some(platform) = config.platforms.get(&platform_name) else {
            eprintln!("warning: platform not found: {platform_name}");
            continue;
        };

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

            for skill in &selected_skills {
                let link_path = target_dir.join(&skill.name);
                match ensure_correct_symlink(&link_path, &skill.dir)? {
                    InstallOutcome::Created => {
                        println!(
                            "created {} -> {}",
                            display_path(&link_path),
                            display_path(&skill.dir)
                        );
                    }
                    InstallOutcome::Skipped => {
                        println!("skipped {}", display_path(&link_path));
                    }
                }
            }
        }
    }

    Ok(())
}
