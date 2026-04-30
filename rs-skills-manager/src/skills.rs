use std::path::{Path, PathBuf};

use crate::error::AppError;

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
}
