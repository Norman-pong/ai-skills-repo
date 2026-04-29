use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::error::AppError;
use crate::path_utils;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillDir {
    pub name: String,
    pub dir: PathBuf,
}

pub fn requires_repo_skills_dir(requested: &[String]) -> bool {
    requested.iter().any(|s| !looks_like_path(s))
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

pub fn resolve_requested_skills(
    repo_skills_dir: Option<&Path>,
    cwd: &Path,
    requested: &[String],
) -> Result<Vec<SkillDir>, AppError> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();

    for raw in requested {
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
            let repo_skills_dir = repo_skills_dir.ok_or_else(|| AppError::RepoSkillsDirInvalid {
                path: cwd.join("skills"),
                source: std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "skills dir not found",
                ),
            })?;
            (raw.clone(), repo_skills_dir.join(raw))
        };

        if !seen.insert(name.clone()) {
            continue;
        }

        validate_skill_dir(raw, &raw_dir)?;
        let dir = std::fs::canonicalize(&raw_dir).map_err(|e| AppError::SkillNotFound {
            skill: raw.clone(),
            path: raw_dir.clone(),
            source: e,
        })?;
        out.push(SkillDir { name, dir });
    }

    Ok(out)
}

fn looks_like_path(raw: &str) -> bool {
    raw.contains('/') || raw.starts_with('.') || raw.starts_with('~')
}

fn validate_skill_dir(skill: &str, dir: &PathBuf) -> Result<(), AppError> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discover_skills_returns_sorted_unique_directory_names() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("b-skill")).unwrap();
        std::fs::create_dir_all(dir.path().join("a-skill")).unwrap();
        std::fs::write(dir.path().join("not-a-dir.txt"), "x").unwrap();

        let skills = discover_skills(dir.path()).unwrap();
        assert_eq!(
            skills.iter().map(|s| s.name.clone()).collect::<Vec<_>>(),
            vec!["a-skill".to_string(), "b-skill".to_string()]
        );
    }

    #[test]
    fn resolve_requested_skills_returns_error_when_skill_missing() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("exists")).unwrap();

        let requested = vec!["missing".to_string()];
        let err = resolve_requested_skills(Some(dir.path()), dir.path(), &requested).unwrap_err();
        match err {
            AppError::SkillNotFound { skill, path, .. } => {
                assert_eq!(skill, "missing");
                assert_eq!(path, dir.path().join("missing"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn resolve_requested_skills_returns_error_when_skill_is_not_directory() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("file-skill"), "x").unwrap();

        let requested = vec!["file-skill".to_string()];
        let err = resolve_requested_skills(Some(dir.path()), dir.path(), &requested).unwrap_err();
        match err {
            AppError::SkillNotFound { skill, path, .. } => {
                assert_eq!(skill, "file-skill");
                assert_eq!(path, dir.path().join("file-skill"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn resolve_requested_skills_dedups_requested_skills_preserving_first_occurrence() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("a")).unwrap();
        std::fs::create_dir_all(dir.path().join("b")).unwrap();

        let requested = vec![
            "a".to_string(),
            "a".to_string(),
            "b".to_string(),
            "b".to_string(),
        ];
        let out = resolve_requested_skills(Some(dir.path()), dir.path(), &requested).unwrap();
        assert_eq!(
            out.iter().map(|s| s.name.clone()).collect::<Vec<_>>(),
            vec!["a".to_string(), "b".to_string()]
        );
    }
}
