use std::process::Command;
use tempfile::TempDir;

fn workspace_root() -> String {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let path = std::path::Path::new(manifest_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    path.to_string_lossy().to_string()
}

fn setup_test_repo() -> TempDir {
    let tmp = TempDir::new().unwrap();
    Command::new("git")
        .args(["init"])
        .current_dir(tmp.path())
        .output()
        .unwrap();
    Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(tmp.path())
        .output()
        .unwrap();
    Command::new("git")
        .args(["config", "user.name", "Test"])
        .current_dir(tmp.path())
        .output()
        .unwrap();
    std::fs::write(tmp.path().join("test.txt"), "hello").unwrap();
    Command::new("git")
        .args(["add", "."])
        .current_dir(tmp.path())
        .output()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "initial commit"])
        .current_dir(tmp.path())
        .output()
        .unwrap();
    tmp
}

fn run_shipforge(args: &[&str], project_dir: &std::path::Path) -> std::process::Output {
    let manifest = format!("{}/Cargo.toml", workspace_root());
    let mut cmd_args = vec!["run", "--manifest-path", &manifest, "-p", "shipforge", "--"];
    cmd_args.extend_from_slice(args);
    Command::new("cargo")
        .args(&cmd_args)
        .current_dir(project_dir)
        .output()
        .unwrap()
}

fn run_codetrail(args: &[&str], project_dir: &std::path::Path) -> std::process::Output {
    let manifest = format!("{}/Cargo.toml", workspace_root());
    let mut cmd_args = vec!["run", "--manifest-path", &manifest, "-p", "codetrail", "--"];
    cmd_args.extend_from_slice(args);
    Command::new("cargo")
        .args(&cmd_args)
        .current_dir(project_dir)
        .output()
        .unwrap()
}

#[test]
fn codetrail_hotspots_works() {
    let tmp = setup_test_repo();
    // Create a file with dependents to generate hotspots
    std::fs::write(tmp.path().join("base.rs"), "pub fn base() {}").unwrap();
    std::fs::write(
        tmp.path().join("user.rs"),
        "use crate::base;\npub fn user() {}",
    )
    .unwrap();
    Command::new("git")
        .args(["add", "."])
        .current_dir(tmp.path())
        .output()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "add base and user"])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    let output = run_codetrail(&["hotspots", "-p", "."], tmp.path());
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Hotspots"));
}

#[test]
fn codetrail_risk_works() {
    let tmp = setup_test_repo();
    // Generate a receipt first so there's data
    let _ = run_shipforge(&["receipt", "-p", "."], tmp.path());

    let output = run_codetrail(&["risk", "-p", "."], tmp.path());
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Risk Summary") || stdout.contains("No activity"));
}

#[test]
fn codetrail_history_works() {
    let tmp = setup_test_repo();
    // Generate a receipt for test.txt
    let _ = run_shipforge(&["receipt", "-p", "."], tmp.path());

    let output = run_codetrail(&["history", "test.txt", "-p", "."], tmp.path());
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Change History"));
}
