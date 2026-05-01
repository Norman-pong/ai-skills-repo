use std::process::Command;

fn bin_path() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_sklink"))
}

#[test]
fn cli_completions_zsh_outputs_script() {
    let out = Command::new(bin_path())
        .args(["completions", "zsh"])
        .output()
        .unwrap();

    assert!(
        out.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(!stdout.trim().is_empty());
    assert!(stdout.contains("sklink"));
}
