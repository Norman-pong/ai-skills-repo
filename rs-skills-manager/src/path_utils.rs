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

#[cfg(test)]
mod tests {
    use std::ffi::OsString;
    use std::sync::Mutex;

    use super::*;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    struct EnvVarGuard {
        key: &'static str,
        old: Option<OsString>,
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            if let Some(old) = self.old.take() {
                std::env::set_var(self.key, old);
            } else {
                std::env::remove_var(self.key);
            }
        }
    }

    fn set_env_var(key: &'static str, value: Option<&str>) -> EnvVarGuard {
        let old = std::env::var_os(key);
        match value {
            Some(v) => std::env::set_var(key, v),
            None => std::env::remove_var(key),
        }
        EnvVarGuard { key, old }
    }

    #[test]
    fn resolve_path_joins_relative_path_to_base_dir() {
        let base = Path::new("/base");
        let out = resolve_path("skills/foo", base).unwrap();
        assert_eq!(out, PathBuf::from("/base/skills/foo"));
    }

    #[test]
    fn resolve_path_keeps_absolute_path() {
        let base = Path::new("/base");
        let out = resolve_path("/abs/path", base).unwrap();
        assert_eq!(out, PathBuf::from("/abs/path"));
    }

    #[test]
    fn resolve_path_expands_tilde_and_then_joins_relative() {
        let _lock = ENV_LOCK.lock().unwrap();
        let _home = set_env_var("HOME", Some("/home/tester"));

        let base = Path::new("/base");
        let out = resolve_path("~/dir/file", base).unwrap();
        assert_eq!(out, PathBuf::from("/home/tester/dir/file"));
    }

    #[test]
    fn resolve_path_expands_bare_tilde() {
        let _lock = ENV_LOCK.lock().unwrap();
        let _home = set_env_var("HOME", Some("/home/tester"));

        let base = Path::new("/base");
        let out = resolve_path("~", base).unwrap();
        assert_eq!(out, PathBuf::from("/home/tester"));
    }

    #[test]
    fn resolve_path_returns_error_when_home_missing_for_tilde() {
        let _lock = ENV_LOCK.lock().unwrap();
        let _home = set_env_var("HOME", None);

        let base = Path::new("/base");
        let err = resolve_path("~/dir", base).unwrap_err();
        assert!(matches!(err, AppError::HomeMissing));
    }
}
