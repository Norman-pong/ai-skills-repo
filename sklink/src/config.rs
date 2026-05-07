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

pub fn load_config_from_path(path: &Path) -> Result<Config, AppError> {
    let content = std::fs::read_to_string(path).map_err(|e| AppError::ConfigRead {
        path: path.to_path_buf(),
        source: e,
    })?;

    toml::from_str::<Config>(&content).map_err(|e| AppError::ConfigParse {
        path: path.to_path_buf(),
        source: e,
    })
}
