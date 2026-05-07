use std::path::Path;
use std::process::Command;

fn bin_path() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_sklink"))
}

fn setup_temp_home() -> tempfile::TempDir {
    tempfile::tempdir().unwrap()
}

fn store_skill_dir(home: &Path, skill: &str) -> std::path::PathBuf {
    home.join(".config/sklink/skills").join(skill)
}

fn other_dir(home: &Path, name: &str) -> std::path::PathBuf {
    home.join("other").join(name)
}

#[test]
fn cli_output_creates_default_dir_and_symlinks_skill() {
    let home = setup_temp_home();
    let project = tempfile::tempdir().unwrap();
    let skill = "software-engineer";

    let stored = store_skill_dir(home.path(), skill);
    std::fs::create_dir_all(&stored).unwrap();
    std::fs::write(stored.join("SKILL.md"), "x").unwrap();

    let out = Command::new(bin_path())
        .env("HOME", home.path())
        .current_dir(project.path())
        .args(["-o", skill])
        .output()
        .unwrap();

    assert!(
        out.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    let link_path = project.path().join(".agent/skills").join(skill);
    let meta = std::fs::symlink_metadata(&link_path).unwrap();
    assert!(meta.file_type().is_symlink());
    let actual = std::fs::canonicalize(&link_path).unwrap();
    let expected = std::fs::canonicalize(&stored).unwrap();
    assert_eq!(actual, expected);
}

#[test]
fn cli_output_export_copies_instead_of_symlink() {
    let home = setup_temp_home();
    let project = tempfile::tempdir().unwrap();
    let skill = "software-engineer";

    let stored = store_skill_dir(home.path(), skill);
    std::fs::create_dir_all(&stored).unwrap();
    std::fs::write(stored.join("SKILL.md"), "x").unwrap();

    let out = Command::new(bin_path())
        .env("HOME", home.path())
        .current_dir(project.path())
        .args(["-o", skill, "--export"])
        .output()
        .unwrap();

    assert!(
        out.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    let path = project
        .path()
        .join(".agent/skills")
        .join(skill)
        .join("SKILL.md");
    let meta = std::fs::symlink_metadata(&path).unwrap();
    assert!(meta.is_file());
    assert!(!meta.file_type().is_symlink());
}

#[test]
fn cli_output_export_dereferences_symlinks_to_files() {
    let home = setup_temp_home();
    let project = tempfile::tempdir().unwrap();
    let skill = "software-engineer";

    let stored = store_skill_dir(home.path(), skill);
    std::fs::create_dir_all(&stored).unwrap();
    std::fs::write(stored.join("real.txt"), "real content").unwrap();
    std::os::unix::fs::symlink(stored.join("real.txt"), stored.join("link.txt")).unwrap();

    let out = Command::new(bin_path())
        .env("HOME", home.path())
        .current_dir(project.path())
        .args(["-o", skill, "--export"])
        .output()
        .unwrap();

    assert!(
        out.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    let exported_link = project
        .path()
        .join(".agent/skills")
        .join(skill)
        .join("link.txt");
    let meta = std::fs::symlink_metadata(&exported_link).unwrap();
    assert!(
        !meta.file_type().is_symlink(),
        "exported symlink should be dereferenced to a regular file"
    );
    assert_eq!(
        std::fs::read_to_string(&exported_link).unwrap(),
        "real content"
    );
}

#[test]
fn cli_output_symlink_fails_when_dest_is_regular_file() {
    let home = setup_temp_home();
    let project = tempfile::tempdir().unwrap();
    let skill = "software-engineer";

    let stored = store_skill_dir(home.path(), skill);
    std::fs::create_dir_all(&stored).unwrap();

    let dest_dir = project.path().join(".agent/skills");
    std::fs::create_dir_all(&dest_dir).unwrap();
    std::fs::write(dest_dir.join(skill), "not a symlink").unwrap();

    let out = Command::new(bin_path())
        .env("HOME", home.path())
        .current_dir(project.path())
        .args(["-o", skill])
        .output()
        .unwrap();

    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("path exists and is not a symlink"));
}

#[test]
fn cli_output_symlink_fails_when_dest_symlink_points_elsewhere() {
    let home = setup_temp_home();
    let project = tempfile::tempdir().unwrap();
    let skill = "software-engineer";

    let stored = store_skill_dir(home.path(), skill);
    std::fs::create_dir_all(&stored).unwrap();

    let dest_dir = project.path().join(".agent/skills");
    std::fs::create_dir_all(&dest_dir).unwrap();
    let other = other_dir(home.path(), "other-skill");
    std::fs::create_dir_all(&other).unwrap();
    std::os::unix::fs::symlink(&other, dest_dir.join(skill)).unwrap();

    let out = Command::new(bin_path())
        .env("HOME", home.path())
        .current_dir(project.path())
        .args(["-o", skill])
        .output()
        .unwrap();

    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("symlink points to a different target"));
}

#[test]
fn cli_output_export_fails_when_dest_exists() {
    let home = setup_temp_home();
    let project = tempfile::tempdir().unwrap();
    let skill = "software-engineer";

    let stored = store_skill_dir(home.path(), skill);
    std::fs::create_dir_all(&stored).unwrap();

    let dest_dir = project.path().join(".agent/skills");
    std::fs::create_dir_all(dest_dir.join(skill)).unwrap();

    let out = Command::new(bin_path())
        .env("HOME", home.path())
        .current_dir(project.path())
        .args(["-o", skill, "--export"])
        .output()
        .unwrap();

    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("output path already exists"));
}

#[test]
fn cli_output_respects_dir_option() {
    let home = setup_temp_home();
    let project = tempfile::tempdir().unwrap();
    let skill = "software-engineer";

    let stored = store_skill_dir(home.path(), skill);
    std::fs::create_dir_all(&stored).unwrap();
    std::fs::write(stored.join("SKILL.md"), "x").unwrap();

    let out = Command::new(bin_path())
        .env("HOME", home.path())
        .current_dir(project.path())
        .args(["-o", skill, "--dir", "custom/skills"])
        .output()
        .unwrap();

    assert!(
        out.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    let link_path = project.path().join("custom/skills").join(skill);
    let meta = std::fs::symlink_metadata(&link_path).unwrap();
    assert!(meta.file_type().is_symlink());
    let actual = std::fs::canonicalize(&link_path).unwrap();
    let expected = std::fs::canonicalize(&stored).unwrap();
    assert_eq!(actual, expected);
}

#[test]
fn cli_output_accepts_long_option() {
    let home = setup_temp_home();
    let project = tempfile::tempdir().unwrap();
    let skill = "software-engineer";

    let stored = store_skill_dir(home.path(), skill);
    std::fs::create_dir_all(&stored).unwrap();
    std::fs::write(stored.join("SKILL.md"), "x").unwrap();

    let out = Command::new(bin_path())
        .env("HOME", home.path())
        .current_dir(project.path())
        .args(["--output", skill])
        .output()
        .unwrap();

    assert!(
        out.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}
