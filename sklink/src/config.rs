use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;

use crate::error::AppError;
use crate::path_utils;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub platforms: HashMap<String, PlatformConfig>,
}

#[derive(Debug, Deserialize)]
pub struct PlatformConfig {
    pub targets: Vec<TargetConfig>,
}

#[derive(Debug, Deserialize)]
pub struct TargetConfig {
    pub dir: String,
}

pub fn load_default_config() -> Result<Config, AppError> {
    let config_path = default_config_path()?;
    load_config_from_path(&config_path)
}

pub fn default_config_path() -> Result<PathBuf, AppError> {
    let raw = "~/.config/sklink/config.toml";
    let cwd = std::env::current_dir().map_err(AppError::Io)?;
    path_utils::resolve_path(raw, &cwd)
}

fn load_config_from_path(path: &Path) -> Result<Config, AppError> {
    let content = std::fs::read_to_string(path).map_err(|e| AppError::ConfigRead {
        path: path.to_path_buf(),
        source: e,
    })?;

    toml::from_str::<Config>(&content).map_err(|e| AppError::ConfigParse {
        path: path.to_path_buf(),
        source: e,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_config_from_path_parses_valid_toml() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.toml");
        std::fs::write(
            &config_path,
            r#"
[platforms.kimi]
targets = [
  { dir = "~/.kimi/skills" },
]
"#,
        )
        .unwrap();

        let config = load_config_from_path(&config_path).unwrap();
        assert!(config.platforms.contains_key("kimi"));
        let platform = config.platforms.get("kimi").unwrap();
        assert_eq!(platform.targets.len(), 1);
        assert_eq!(platform.targets[0].dir, "~/.kimi/skills");
    }

    #[test]
    fn load_config_from_path_returns_read_error_when_missing() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("missing.toml");

        let err = load_config_from_path(&config_path).unwrap_err();
        match err {
            AppError::ConfigRead { path, .. } => assert_eq!(path, config_path),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn load_config_from_path_returns_parse_error_for_invalid_toml() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.toml");
        std::fs::write(&config_path, "not_toml = [").unwrap();

        let err = load_config_from_path(&config_path).unwrap_err();
        match err {
            AppError::ConfigParse { path, .. } => assert_eq!(path, config_path),
            other => panic!("unexpected error: {other:?}"),
        }
    }
}
