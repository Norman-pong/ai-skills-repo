use sklink::config::load_config_from_path;
use sklink::error::AppError;

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
