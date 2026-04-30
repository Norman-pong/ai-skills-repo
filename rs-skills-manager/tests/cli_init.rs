use std::path::Path;
use std::process::Command;

fn bin_path() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_sklink"))
}

fn setup_temp_home() -> tempfile::TempDir {
    tempfile::tempdir().unwrap()
}

fn make_repo_skills_dir(root: &Path) -> std::path::PathBuf {
    let repo_skills_dir = root.join("repo_skills");
    std::fs::create_dir_all(&repo_skills_dir).unwrap();
    repo_skills_dir
}

fn config_path(home: &Path) -> std::path::PathBuf {
    home.join(".config/rs-skills-manager/config.toml")
}

#[test]
fn cli_init_creates_default_config() {
    let home = setup_temp_home();
    let repo_root = tempfile::tempdir().unwrap();
    let _repo_skills_dir = make_repo_skills_dir(repo_root.path());

    let out = Command::new(bin_path())
        .env("HOME", home.path())
        .arg("init")
        .output()
        .unwrap();

    assert!(
        out.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    let cfg_path = config_path(home.path());
    let content = std::fs::read_to_string(cfg_path).unwrap();
    assert!(content.contains("[platforms.kimi]"));
    assert!(content.contains(r#"{ dir = "~/.kimi/skills" }"#));
    assert!(content.contains("[platforms.trae]"));
    assert!(content.contains(r#"{ dir = "~/.trae/skills" }"#));
}

#[test]
fn cli_init_fails_when_config_exists_without_force() {
    let home = setup_temp_home();
    let repo_root = tempfile::tempdir().unwrap();
    let _repo_skills_dir = make_repo_skills_dir(repo_root.path());

    let out1 = Command::new(bin_path())
        .env("HOME", home.path())
        .arg("init")
        .output()
        .unwrap();
    assert!(out1.status.success());

    let out2 = Command::new(bin_path())
        .env("HOME", home.path())
        .arg("init")
        .output()
        .unwrap();
    assert!(!out2.status.success());
    let stderr = String::from_utf8_lossy(&out2.stderr);
    assert!(stderr.contains("config already exists"));
}
