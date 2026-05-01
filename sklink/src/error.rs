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

    #[error("skill already exists in local store (use --force to overwrite): {skill}: {path}")]
    StoreSkillAlreadyExists { skill: String, path: PathBuf },

    #[error("failed to backup existing local store skill: {from} -> {to}: {source}")]
    StoreSkillBackup {
        from: PathBuf,
        to: PathBuf,
        source: std::io::Error,
    },

    #[error("failed to create directory in local store: {dir}: {source}")]
    StoreDirCreate {
        dir: PathBuf,
        source: std::io::Error,
    },

    #[error("failed to list directory in local store: {dir}: {source}")]
    StoreDirRead {
        dir: PathBuf,
        source: std::io::Error,
    },

    #[error("failed to copy file into local store: {from} -> {to}: {source}")]
    StoreFileCopy {
        from: PathBuf,
        to: PathBuf,
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
