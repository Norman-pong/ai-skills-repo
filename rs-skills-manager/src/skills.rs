use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::error::AppError;

pub fn discover_skills(repo_skills_dir: &Path) -> Result<Vec<String>, AppError> {
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
        skills.push(name);
    }

    skills.sort();
    skills.dedup();
    Ok(skills)
}

pub fn validate_skills(
    repo_skills_dir: &Path,
    requested: &[String],
) -> Result<Vec<String>, AppError> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();

    for skill in requested {
        if !seen.insert(skill.clone()) {
            continue;
        }
        let path = repo_skills_dir.join(skill);
        validate_skill_dir(skill, &path)?;
        out.push(skill.clone());
    }

    Ok(out)
}

fn validate_skill_dir(skill: &str, path: &PathBuf) -> Result<(), AppError> {
    let meta = std::fs::metadata(path).map_err(|e| AppError::SkillNotFound {
        skill: skill.to_string(),
        path: path.clone(),
        source: e,
    })?;
    if !meta.is_dir() {
        return Err(AppError::SkillNotFound {
            skill: skill.to_string(),
            path: path.clone(),
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
        assert_eq!(skills, vec!["a-skill".to_string(), "b-skill".to_string()]);
    }

    #[test]
    fn validate_skills_returns_error_when_skill_missing() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("exists")).unwrap();

        let requested = vec!["missing".to_string()];
        let err = validate_skills(dir.path(), &requested).unwrap_err();
        match err {
            AppError::SkillNotFound { skill, path, .. } => {
                assert_eq!(skill, "missing");
                assert_eq!(path, dir.path().join("missing"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn validate_skills_returns_error_when_skill_is_not_directory() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("file-skill"), "x").unwrap();

        let requested = vec!["file-skill".to_string()];
        let err = validate_skills(dir.path(), &requested).unwrap_err();
        match err {
            AppError::SkillNotFound { skill, path, .. } => {
                assert_eq!(skill, "file-skill");
                assert_eq!(path, dir.path().join("file-skill"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn validate_skills_dedups_requested_skills_preserving_first_occurrence() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("a")).unwrap();
        std::fs::create_dir_all(dir.path().join("b")).unwrap();

        let requested = vec![
            "a".to_string(),
            "a".to_string(),
            "b".to_string(),
            "b".to_string(),
        ];
        let out = validate_skills(dir.path(), &requested).unwrap();
        assert_eq!(out, vec!["a".to_string(), "b".to_string()]);
    }
}
