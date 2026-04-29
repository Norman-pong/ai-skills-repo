use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("failed to read config: {path}: {source}")]
    ConfigRead {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("failed to parse config: {path}: {source}")]
    ConfigParse {
        path: PathBuf,
        source: toml::de::Error,
    },

    #[error("HOME is not set; cannot expand ~")]
    HomeMissing,

    #[error("platform not found: {platform}")]
    PlatformNotFound { platform: String },

    #[error("invalid repo_skills_dir: {path}: {source}")]
    RepoSkillsDirInvalid {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("failed to list skills in repo_skills_dir: {dir}: {source}")]
    RepoSkillsDirRead {
        dir: PathBuf,
        source: std::io::Error,
    },

    #[error("skill not found: {skill}: {path}: {source}")]
    SkillNotFound {
        skill: String,
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("failed to create directory: {dir}: {source}")]
    CreateDir {
        dir: PathBuf,
        source: std::io::Error,
    },

    #[error("path exists and is not a symlink: {path}")]
    LinkPathNotSymlink { path: PathBuf },

    #[error("symlink points to a different target: {path} -> {actual} (expected {expected})")]
    LinkPathWrongTarget {
        path: PathBuf,
        actual: PathBuf,
        expected: PathBuf,
    },

    #[error("failed to read symlink: {path}: {source}")]
    ReadLink {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("failed to create symlink: {path} -> {target}: {source}")]
    CreateSymlink {
        path: PathBuf,
        target: PathBuf,
        source: std::io::Error,
    },

    #[error("config already exists: {path} (use --force to overwrite)")]
    ConfigAlreadyExists { path: PathBuf },

    #[error("failed to write config: {path}: {source}")]
    ConfigWrite {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error(transparent)]
    Io(#[from] std::io::Error),
}
