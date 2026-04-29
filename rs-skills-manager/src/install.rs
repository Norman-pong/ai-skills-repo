use std::path::{Path, PathBuf};

use crate::error::AppError;

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

fn resolve_symlink_target(link_path: &Path, raw_target: &PathBuf) -> PathBuf {
    if raw_target.is_absolute() {
        raw_target.clone()
    } else {
        let parent = link_path.parent().unwrap_or_else(|| Path::new("."));
        parent.join(raw_target)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn canonical(path: &Path) -> PathBuf {
        std::fs::canonicalize(path).unwrap()
    }

    #[test]
    fn ensure_correct_symlink_creates_when_missing() {
        let dir = tempfile::tempdir().unwrap();
        let link_path = dir.path().join("link");
        let link_target = dir.path().join("target");
        std::fs::create_dir_all(&link_target).unwrap();

        let outcome = ensure_correct_symlink(&link_path, &link_target).unwrap();
        assert_eq!(outcome, InstallOutcome::Created);

        let meta = std::fs::symlink_metadata(&link_path).unwrap();
        assert!(meta.file_type().is_symlink());

        let raw = std::fs::read_link(&link_path).unwrap();
        let resolved = resolve_symlink_target(&link_path, &raw);
        assert_eq!(canonical(&resolved), canonical(&link_target));
    }

    #[test]
    fn ensure_correct_symlink_skips_when_correct() {
        let dir = tempfile::tempdir().unwrap();
        let link_path = dir.path().join("link");
        let link_target = dir.path().join("target");
        std::fs::create_dir_all(&link_target).unwrap();

        ensure_correct_symlink(&link_path, &link_target).unwrap();
        let outcome = ensure_correct_symlink(&link_path, &link_target).unwrap();
        assert_eq!(outcome, InstallOutcome::Skipped);
    }

    #[test]
    fn ensure_correct_symlink_errors_when_existing_path_is_not_symlink() {
        let dir = tempfile::tempdir().unwrap();
        let link_path = dir.path().join("link");
        let link_target = dir.path().join("target");
        std::fs::create_dir_all(&link_target).unwrap();
        std::fs::write(&link_path, "not a symlink").unwrap();

        let err = ensure_correct_symlink(&link_path, &link_target).unwrap_err();
        match err {
            AppError::LinkPathNotSymlink { path } => assert_eq!(path, link_path),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn ensure_correct_symlink_errors_when_symlink_points_elsewhere() {
        let dir = tempfile::tempdir().unwrap();
        let link_path = dir.path().join("link");
        let expected = dir.path().join("expected");
        let other = dir.path().join("other");
        std::fs::create_dir_all(&expected).unwrap();
        std::fs::create_dir_all(&other).unwrap();

        std::os::unix::fs::symlink(&other, &link_path).unwrap();
        let err = ensure_correct_symlink(&link_path, &expected).unwrap_err();
        match err {
            AppError::LinkPathWrongTarget {
                path,
                actual,
                expected,
            } => {
                assert_eq!(path, link_path);
                assert_eq!(actual, canonical(&other));
                assert_eq!(expected, canonical(&dir.path().join("expected")));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn ensure_correct_symlink_accepts_relative_symlink_target() {
        let dir = tempfile::tempdir().unwrap();
        let repo = dir.path().join("repo");
        let targets = dir.path().join("targets");
        std::fs::create_dir_all(&repo).unwrap();
        std::fs::create_dir_all(&targets).unwrap();

        let link_target = repo.join("skill");
        std::fs::create_dir_all(&link_target).unwrap();

        let link_path = targets.join("skill");
        std::os::unix::fs::symlink("../repo/skill", &link_path).unwrap();

        let outcome = ensure_correct_symlink(&link_path, &link_target).unwrap();
        assert_eq!(outcome, InstallOutcome::Skipped);
    }
}
