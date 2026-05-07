use std::path::{Path, PathBuf};

use crate::error::AppError;

pub fn resolve_path(raw: &str, base_dir: &Path) -> Result<PathBuf, AppError> {
    let expanded = expand_tilde(raw)?;
    if expanded.is_relative() {
        Ok(base_dir.join(expanded))
    } else {
        Ok(expanded)
    }
}

fn expand_tilde(raw: &str) -> Result<PathBuf, AppError> {
    if raw == "~" {
        return home_dir();
    }

    if let Some(rest) = raw.strip_prefix("~/") {
        return Ok(home_dir()?.join(rest));
    }

    Ok(PathBuf::from(raw))
}

fn home_dir() -> Result<PathBuf, AppError> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or(AppError::HomeMissing)
}
