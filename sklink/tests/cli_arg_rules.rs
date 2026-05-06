use std::process::Command;

fn bin_path() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_sklink"))
}

fn setup_temp_home() -> tempfile::TempDir {
    tempfile::tempdir().unwrap()
}

#[test]
fn cli_rejects_platform_without_async() {
    let home = setup_temp_home();
    let cwd = tempfile::tempdir().unwrap();

    let out = Command::new(bin_path())
        .env("HOME", home.path())
        .current_dir(cwd.path())
        .args(["-p", "kimi"])
        .output()
        .unwrap();

    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("--async"));
}

#[test]
fn cli_rejects_force_without_install_sources() {
    let home = setup_temp_home();
    let cwd = tempfile::tempdir().unwrap();

    let out = Command::new(bin_path())
        .env("HOME", home.path())
        .current_dir(cwd.path())
        .arg("--force")
        .output()
        .unwrap();

    assert!(!out.status.success());
}

#[test]
fn cli_rejects_dir_without_output() {
    let home = setup_temp_home();
    let cwd = tempfile::tempdir().unwrap();

    let out = Command::new(bin_path())
        .env("HOME", home.path())
        .current_dir(cwd.path())
        .args(["--dir", "x"])
        .output()
        .unwrap();

    assert!(!out.status.success());
}

#[test]
fn cli_rejects_export_without_output() {
    let home = setup_temp_home();
    let cwd = tempfile::tempdir().unwrap();

    let out = Command::new(bin_path())
        .env("HOME", home.path())
        .current_dir(cwd.path())
        .arg("--export")
        .output()
        .unwrap();

    assert!(!out.status.success());
}

#[test]
fn cli_rejects_output_with_async() {
    let home = setup_temp_home();
    let cwd = tempfile::tempdir().unwrap();

    let out = Command::new(bin_path())
        .env("HOME", home.path())
        .current_dir(cwd.path())
        .args(["-o", "software-engineer", "--async"])
        .output()
        .unwrap();

    assert!(!out.status.success());
}

#[test]
fn cli_rejects_output_with_install() {
    let home = setup_temp_home();
    let cwd = tempfile::tempdir().unwrap();

    let out = Command::new(bin_path())
        .env("HOME", home.path())
        .current_dir(cwd.path())
        .args([
            "-o",
            "software-engineer",
            "-i",
            "./skills/software-engineer",
        ])
        .output()
        .unwrap();

    assert!(!out.status.success());
}
