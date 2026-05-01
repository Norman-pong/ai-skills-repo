use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

fn bin_path() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_sklink"))
}

fn setup_temp_home() -> TempDir {
    tempfile::tempdir().unwrap()
}

fn store_skill_dir(home: &Path, skill: &str) -> std::path::PathBuf {
    home.join(".config/sklink/skills").join(skill)
}

fn write_config(home: &Path, target_dir: &Path) {
    let config_dir = home.join(".config/sklink");
    std::fs::create_dir_all(&config_dir).unwrap();
    let config_path = config_dir.join("config.toml");
    let content = format!(
        r#"
[platforms.kimi]
targets = [
  {{ dir = "{target_dir}" }},
]
"#,
        target_dir = target_dir.display()
    );
    std::fs::write(config_path, content).unwrap();
}

fn make_repo_root_with_skills(root: &Path, skill: &str) -> std::path::PathBuf {
    let repo_root = root.join("repo_root");
    std::fs::create_dir_all(repo_root.join("skills").join(skill)).unwrap();
    repo_root
}

fn run_cli(home: &Path, repo_root: &Path, args: &[&str]) -> std::process::Output {
    run_cli_in_dir(home, repo_root, args)
}

fn run_cli_in_dir(home: &Path, cwd: &Path, args: &[&str]) -> std::process::Output {
    Command::new(bin_path())
        .args(args)
        .env("HOME", home)
        .current_dir(cwd)
        .output()
        .unwrap()
}

#[test]
fn cli_installs_symlink_into_target_dir_and_is_idempotent() {
    let home = setup_temp_home();
    let tmp = tempfile::tempdir().unwrap();
    let skill = "software-engineer";
    let repo_root = make_repo_root_with_skills(tmp.path(), skill);
    let target_dir = home.path().join("targets/skills");

    write_config(home.path(), &target_dir);
    std::fs::create_dir_all(&target_dir).unwrap();

    let out1 = run_cli(home.path(), &repo_root, &["-i", skill, "-p", "kimi"]);
    assert!(
        out1.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out1.stdout),
        String::from_utf8_lossy(&out1.stderr)
    );

    let link_path = target_dir.join(skill);
    let meta = std::fs::symlink_metadata(&link_path).unwrap();
    assert!(meta.file_type().is_symlink());
    let actual = std::fs::canonicalize(&link_path).unwrap();
    let expected = std::fs::canonicalize(store_skill_dir(home.path(), skill)).unwrap();
    assert_eq!(actual, expected);

    let other_cwd = tempfile::tempdir().unwrap();
    let out2 = run_cli_in_dir(home.path(), other_cwd.path(), &["-i", skill, "-p", "kimi"]);
    assert!(
        out2.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out2.stdout),
        String::from_utf8_lossy(&out2.stderr)
    );
    let stdout2 = String::from_utf8_lossy(&out2.stdout);
    assert!(stdout2.contains("skipped"));
}

#[test]
fn cli_accepts_install_long_option() {
    let home = setup_temp_home();
    let tmp = tempfile::tempdir().unwrap();
    let skill = "software-engineer";
    let repo_root = make_repo_root_with_skills(tmp.path(), skill);
    let target_dir = home.path().join("targets/skills");

    write_config(home.path(), &target_dir);
    std::fs::create_dir_all(&target_dir).unwrap();

    let out = run_cli(
        home.path(),
        &repo_root,
        &["--install", skill, "--platform", "kimi"],
    );
    assert!(
        out.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    let link_path = target_dir.join(skill);
    let actual = std::fs::canonicalize(&link_path).unwrap();
    let expected = std::fs::canonicalize(store_skill_dir(home.path(), skill)).unwrap();
    assert_eq!(actual, expected);
}

#[test]
fn cli_can_run_from_skills_dir_using_skill_name() {
    let home = setup_temp_home();
    let tmp = tempfile::tempdir().unwrap();
    let skill = "software-engineer";
    let repo_root = make_repo_root_with_skills(tmp.path(), skill);
    let target_dir = home.path().join("targets/skills");

    write_config(home.path(), &target_dir);
    std::fs::create_dir_all(&target_dir).unwrap();

    let out = run_cli_in_dir(
        home.path(),
        &repo_root.join("skills"),
        &["-i", skill, "-p", "kimi"],
    );
    assert!(
        out.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    let link_path = target_dir.join(skill);
    let actual = std::fs::canonicalize(&link_path).unwrap();
    let expected = std::fs::canonicalize(store_skill_dir(home.path(), skill)).unwrap();
    assert_eq!(actual, expected);
}

#[test]
fn cli_accepts_skill_path_without_repo_skills_dir_in_cwd() {
    let home = setup_temp_home();
    let tmp = tempfile::tempdir().unwrap();
    let skill = "software-engineer";
    let repo_root = make_repo_root_with_skills(tmp.path(), skill);
    let target_dir = home.path().join("targets/skills");

    write_config(home.path(), &target_dir);
    std::fs::create_dir_all(&target_dir).unwrap();

    let other_cwd = tempfile::tempdir().unwrap();
    let skill_dir = repo_root.join("skills").join(skill);
    let skill_dir = std::fs::canonicalize(&skill_dir).unwrap();

    let out = Command::new(bin_path())
        .env("HOME", home.path())
        .current_dir(other_cwd.path())
        .args(["-i", skill_dir.to_str().unwrap(), "-p", "kimi"])
        .output()
        .unwrap();

    assert!(
        out.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    let link_path = target_dir.join(skill);
    let actual = std::fs::canonicalize(&link_path).unwrap();
    let expected = std::fs::canonicalize(store_skill_dir(home.path(), skill)).unwrap();
    assert_eq!(actual, expected);
}

#[test]
fn cli_warns_when_platform_not_found() {
    let home = setup_temp_home();
    let tmp = tempfile::tempdir().unwrap();
    let skill = "software-engineer";
    let repo_root = make_repo_root_with_skills(tmp.path(), skill);
    let target_dir = home.path().join("targets/skills");
    write_config(home.path(), &target_dir);

    let out = run_cli(
        home.path(),
        &repo_root,
        &["-i", skill, "-p", "missing-platform"],
    );
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("platform not found"));
    assert!(stderr.contains("available platforms"));
    assert!(stderr.contains("kimi"));
}

#[test]
fn cli_fails_when_config_missing() {
    let home = setup_temp_home();
    let tmp = tempfile::tempdir().unwrap();
    let repo_root = tmp.path();
    let out = run_cli(home.path(), repo_root, &["-p", "all"]);
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("failed to read config"));
}

#[test]
fn cli_fails_when_target_path_conflicts_with_regular_file() {
    let home = setup_temp_home();
    let tmp = tempfile::tempdir().unwrap();
    let skill = "software-engineer";
    let repo_root = make_repo_root_with_skills(tmp.path(), skill);
    let target_dir = home.path().join("targets/skills");
    write_config(home.path(), &target_dir);

    std::fs::create_dir_all(&target_dir).unwrap();
    std::fs::write(target_dir.join(skill), "not a symlink").unwrap();

    let out = run_cli(home.path(), &repo_root, &["-i", skill, "-p", "kimi"]);
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("path exists and is not a symlink"));
}

#[test]
fn cli_fails_when_symlink_points_to_different_target() {
    let home = setup_temp_home();
    let tmp = tempfile::tempdir().unwrap();
    let skill = "software-engineer";
    let repo_root = make_repo_root_with_skills(tmp.path(), skill);
    let target_dir = home.path().join("targets/skills");
    write_config(home.path(), &target_dir);

    std::fs::create_dir_all(&target_dir).unwrap();
    let other = tmp.path().join("other-skill");
    std::fs::create_dir_all(&other).unwrap();
    std::os::unix::fs::symlink(&other, target_dir.join(skill)).unwrap();

    let out = run_cli(home.path(), &repo_root, &["-i", skill, "-p", "kimi"]);
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("symlink points to a different target"));
}

#[test]
fn cli_fails_when_store_skill_exists_without_force() {
    let home = setup_temp_home();
    let tmp = tempfile::tempdir().unwrap();
    let skill = "software-engineer";
    let repo_root = make_repo_root_with_skills(tmp.path(), skill);
    let target_dir = home.path().join("targets/skills");

    write_config(home.path(), &target_dir);
    std::fs::create_dir_all(&target_dir).unwrap();

    let existing = store_skill_dir(home.path(), skill);
    std::fs::create_dir_all(&existing).unwrap();

    let source = repo_root.join("skills").join(skill);
    let out = Command::new(bin_path())
        .env("HOME", home.path())
        .current_dir(&repo_root)
        .args(["-i", source.to_str().unwrap(), "-p", "kimi"])
        .output()
        .unwrap();
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("skill already exists in local store"));
}

#[test]
fn cli_force_overwrites_store_skill_by_backup_and_recopy() {
    let home = setup_temp_home();
    let tmp = tempfile::tempdir().unwrap();
    let skill = "software-engineer";
    let repo_root = make_repo_root_with_skills(tmp.path(), skill);
    let target_dir = home.path().join("targets/skills");

    let existing = store_skill_dir(home.path(), skill);
    std::fs::create_dir_all(&existing).unwrap();
    std::fs::write(existing.join("old.txt"), "old").unwrap();

    std::fs::write(repo_root.join("skills").join(skill).join("new.txt"), "new").unwrap();

    write_config(home.path(), &target_dir);
    std::fs::create_dir_all(&target_dir).unwrap();

    let out = run_cli(
        home.path(),
        &repo_root,
        &["--force", "-i", skill, "-p", "kimi"],
    );
    assert!(
        out.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    let stored = store_skill_dir(home.path(), skill);
    assert!(stored.join("new.txt").exists());

    let backups = home.path().join(".config/sklink/backups").join(skill);
    let entries = std::fs::read_dir(&backups).unwrap().collect::<Vec<_>>();
    assert!(!entries.is_empty());
}

#[test]
fn cli_skips_when_target_dir_missing() {
    let home = setup_temp_home();
    let tmp = tempfile::tempdir().unwrap();
    let skill = "software-engineer";
    let repo_root = make_repo_root_with_skills(tmp.path(), skill);
    let target_dir = home.path().join("targets/missing");

    write_config(home.path(), &target_dir);

    let out = run_cli(home.path(), &repo_root, &["-i", skill, "-p", "kimi"]);
    assert!(out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("target dir not found"));
    assert!(!target_dir.join(skill).exists());
}
