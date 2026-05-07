use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::AppError;
use crate::skills;
use crate::store;

#[derive(Debug)]
pub struct TempDir {
    path: PathBuf,
}

impl TempDir {
    pub fn new(prefix: &str) -> Result<Self, AppError> {
        let mut base = std::env::temp_dir();
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| AppError::Io(std::io::Error::other(e)))?
            .as_millis();
        let pid = std::process::id();
        base.push(format!("sklink_{prefix}_{pid}_{ts}"));
        std::fs::create_dir_all(&base).map_err(|e| AppError::CreateDir {
            dir: base.clone(),
            source: e,
        })?;
        Ok(Self { path: base })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

pub fn looks_like_git_url(raw: &str) -> bool {
    raw.contains("github.com") || raw.starts_with("git@") || raw.ends_with(".git")
}

pub fn stage_from_git_url(
    url: &str,
    store_dir: &Path,
    cwd: &Path,
    force: bool,
) -> Result<Vec<skills::SkillDir>, AppError> {
    ensure_git_available()?;

    let tmp = TempDir::new("git")?;
    let repo_dir = tmp.path().join("repo");

    let output = Command::new("git")
        .arg("clone")
        .arg("--depth")
        .arg("1")
        .arg(url)
        .arg(&repo_dir)
        .output();

    let output = match output {
        Ok(o) => o,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            return Err(AppError::GitNotAvailable)
        }
        Err(err) => return Err(AppError::Io(err)),
    };

    if !output.status.success() {
        let mut msg = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if msg.is_empty() {
            msg = format!("exit status: {}", output.status);
        }
        return Err(AppError::GitCloneFailed {
            url: url.to_string(),
            message: msg,
        });
    }

    stage_from_git_repo_dir(url, &repo_dir, store_dir, cwd, force)
}

fn stage_from_git_repo_dir(
    url: &str,
    repo_dir: &Path,
    store_dir: &Path,
    cwd: &Path,
    force: bool,
) -> Result<Vec<skills::SkillDir>, AppError> {
    let skills_dir = repo_dir.join("skills");
    if skills_dir.is_dir() {
        let found = skills::discover_skills(&skills_dir)?;
        let mut staged = Vec::new();
        for skill in found {
            let dir = store::stage_skill_to_store(store_dir, &skill.name, &skill.dir, force)?;
            staged.push(skills::SkillDir {
                name: skill.name,
                dir: std::fs::canonicalize(dir).map_err(AppError::Io)?,
            });
        }
        return Ok(staged);
    }

    let name = repo_name_from_url(url, cwd)?;
    let dir = store::stage_skill_to_store(store_dir, &name, repo_dir, force)?;
    Ok(vec![skills::SkillDir {
        name,
        dir: std::fs::canonicalize(dir).map_err(AppError::Io)?,
    }])
}

fn ensure_git_available() -> Result<(), AppError> {
    let output = Command::new("git").arg("--version").output();
    match output {
        Ok(o) if o.status.success() => Ok(()),
        Ok(_) => Err(AppError::GitNotAvailable),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Err(AppError::GitNotAvailable),
        Err(err) => Err(AppError::Io(err)),
    }
}

fn repo_name_from_url(url: &str, cwd: &Path) -> Result<String, AppError> {
    let _ = cwd;
    let trimmed = url.trim_end_matches('/');
    let last = trimmed.rsplit(['/', ':']).next().unwrap_or("repo").trim();
    let last = last.strip_suffix(".git").unwrap_or(last);
    if last.is_empty() {
        return Ok("repo".to_string());
    }
    Ok(last.to_string())
}
