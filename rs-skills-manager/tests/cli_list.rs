use std::process::Command;

fn bin_path() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_sklink"))
}

fn setup_temp_home() -> tempfile::TempDir {
    tempfile::tempdir().unwrap()
}

fn store_dir(home: &std::path::Path) -> std::path::PathBuf {
    home.join(".config/rs-skills-manager/skills")
}

#[test]
fn cli_list_prints_sorted_skills_from_repo_root() {
    let home = setup_temp_home();
    let tmp = tempfile::tempdir().unwrap();
    let repo_root = tmp.path().join("repo_root");
    std::fs::create_dir_all(repo_root.join("skills").join("b")).unwrap();
    std::fs::create_dir_all(repo_root.join("skills").join("a")).unwrap();

    let out = Command::new(bin_path())
        .env("HOME", home.path())
        .current_dir(&repo_root)
        .arg("list")
        .output()
        .unwrap();

    assert!(
        out.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(stdout.lines().collect::<Vec<_>>(), vec!["a", "b"]);
}

#[test]
fn cli_list_can_run_from_skills_dir() {
    let home = setup_temp_home();
    let tmp = tempfile::tempdir().unwrap();
    let repo_root = tmp.path().join("repo_root");
    std::fs::create_dir_all(repo_root.join("skills").join("b")).unwrap();
    std::fs::create_dir_all(repo_root.join("skills").join("a")).unwrap();

    let out = Command::new(bin_path())
        .env("HOME", home.path())
        .current_dir(repo_root.join("skills"))
        .arg("list")
        .output()
        .unwrap();

    assert!(
        out.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(stdout.lines().collect::<Vec<_>>(), vec!["a", "b"]);
}

#[test]
fn cli_list_falls_back_to_local_store_when_repo_skills_dir_missing() {
    let home = setup_temp_home();
    let cwd = tempfile::tempdir().unwrap();

    let store_dir = store_dir(home.path());
    std::fs::create_dir_all(store_dir.join("b")).unwrap();
    std::fs::create_dir_all(store_dir.join("a")).unwrap();

    let out = Command::new(bin_path())
        .env("HOME", home.path())
        .current_dir(cwd.path())
        .arg("list")
        .output()
        .unwrap();

    assert!(
        out.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(stdout.lines().collect::<Vec<_>>(), vec!["a", "b"]);
}

#[test]
fn cli_list_installed_prints_target_entries() {
    let home = setup_temp_home();
    let tmp = tempfile::tempdir().unwrap();
    let target_dir = tmp.path().join("targets/skills");
    std::fs::create_dir_all(&target_dir).unwrap();

    let config_dir = home.path().join(".config/rs-skills-manager");
    std::fs::create_dir_all(&config_dir).unwrap();
    std::fs::write(
        config_dir.join("config.toml"),
        format!(
            r#"
[platforms.kimi]
targets = [
  {{ dir = "{}" }},
]
"#,
            target_dir.display()
        ),
    )
    .unwrap();

    let store_dir = store_dir(home.path());
    let skill = "software-engineer";
    std::fs::create_dir_all(store_dir.join(skill)).unwrap();
    std::os::unix::fs::symlink(store_dir.join(skill), target_dir.join(skill)).unwrap();

    std::fs::write(target_dir.join(".DS_Store"), "x").unwrap();
    std::fs::write(target_dir.join("notes.txt"), "x").unwrap();
    std::fs::create_dir_all(target_dir.join("dir-skill")).unwrap();

    let out = Command::new(bin_path())
        .env("HOME", home.path())
        .current_dir(tmp.path())
        .args(["list", "--installed"])
        .output()
        .unwrap();

    assert!(
        out.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("kimi"));
    assert!(stdout.contains(target_dir.to_string_lossy().as_ref()));
    assert!(stdout.contains(&format!("[L] {skill} ->")));
    assert!(stdout.contains("(ok)"));
    assert!(stdout.contains("[F] notes.txt"));
    assert!(stdout.contains("[D] dir-skill"));
    assert!(!stdout.contains(".DS_Store"));
}
