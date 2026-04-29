use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

fn bin_path() -> std::path::PathBuf {
    let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("target");
    p.push("debug");
    p.push("rs-skills-manager");
    if cfg!(windows) {
        p.set_extension("exe");
    }
    p
}

fn setup_temp_home() -> TempDir {
    tempfile::tempdir().unwrap()
}

fn write_config(home: &Path, target_dir: &Path) {
    let config_dir = home.join(".config/rs-skills-manager");
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

    let out1 = run_cli(home.path(), &repo_root, &["-i", skill, "-o", "kimi"]);
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
    let expected = std::fs::canonicalize(repo_root.join("skills").join(skill)).unwrap();
    assert_eq!(actual, expected);

    let out2 = run_cli(home.path(), &repo_root, &["-i", skill, "-o", "kimi"]);
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
fn cli_can_run_from_skills_dir_using_skill_name() {
    let home = setup_temp_home();
    let tmp = tempfile::tempdir().unwrap();
    let skill = "software-engineer";
    let repo_root = make_repo_root_with_skills(tmp.path(), skill);
    let target_dir = home.path().join("targets/skills");

    write_config(home.path(), &target_dir);

    let out = run_cli_in_dir(
        home.path(),
        &repo_root.join("skills"),
        &["-i", skill, "-o", "kimi"],
    );
    assert!(
        out.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    let link_path = target_dir.join(skill);
    let actual = std::fs::canonicalize(&link_path).unwrap();
    let expected = std::fs::canonicalize(repo_root.join("skills").join(skill)).unwrap();
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

    let other_cwd = tempfile::tempdir().unwrap();
    let skill_dir = repo_root.join("skills").join(skill);
    let skill_dir = std::fs::canonicalize(&skill_dir).unwrap();

    let out = Command::new(bin_path())
        .env("HOME", home.path())
        .current_dir(other_cwd.path())
        .args(["-i", skill_dir.to_str().unwrap(), "-o", "kimi"])
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
    assert_eq!(actual, skill_dir);
}

#[test]
fn cli_fails_when_platform_not_found() {
    let home = setup_temp_home();
    let tmp = tempfile::tempdir().unwrap();
    let skill = "software-engineer";
    let repo_root = make_repo_root_with_skills(tmp.path(), skill);
    let target_dir = home.path().join("targets/skills");
    write_config(home.path(), &target_dir);

    let out = run_cli(
        home.path(),
        &repo_root,
        &["-i", skill, "-o", "missing-platform"],
    );
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("platform not found"));
}

#[test]
fn cli_fails_when_config_missing() {
    let home = setup_temp_home();
    let tmp = tempfile::tempdir().unwrap();
    let repo_root = tmp.path();
    let out = run_cli(home.path(), repo_root, &["-o", "all"]);
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

    let out = run_cli(home.path(), &repo_root, &["-i", skill, "-o", "kimi"]);
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

    let out = run_cli(home.path(), &repo_root, &["-i", skill, "-o", "kimi"]);
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("symlink points to a different target"));
}
