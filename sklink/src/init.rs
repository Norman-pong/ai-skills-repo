use std::path::{Path, PathBuf};

use crate::error::AppError;

pub struct InitOptions {
    pub force: bool,
}

pub fn init_config(_cwd: &Path, opts: InitOptions) -> Result<PathBuf, AppError> {
    let config_path = crate::config::default_config_path()?;
    if config_path.exists() && !opts.force {
        return Err(AppError::ConfigAlreadyExists { path: config_path });
    }

    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| AppError::CreateDir {
            dir: parent.to_path_buf(),
            source: e,
        })?;
    }

    let content = render_default_config();
    std::fs::write(&config_path, content).map_err(|e| AppError::ConfigWrite {
        path: config_path.clone(),
        source: e,
    })?;

    Ok(config_path)
}

pub fn render_default_config() -> String {
    r#"[platforms.kimi]
targets = [
  { dir = "~/.kimi/skills" },
]

[platforms.trae]
targets = [
  { dir = "~/.trae/skills" },
]
"#
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_default_config_contains_default_platforms() {
        let cfg = render_default_config();
        assert!(cfg.contains(r#"[platforms.kimi]"#));
        assert!(cfg.contains(r#"{ dir = "~/.kimi/skills" }"#));
        assert!(cfg.contains(r#"[platforms.trae]"#));
        assert!(cfg.contains(r#"{ dir = "~/.trae/skills" }"#));
    }
}
