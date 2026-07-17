use std::process::Command;
use tempfile::TempDir;

fn workspace_root() -> String {
    // The integration test binary is in crates/shipforge/tests/../../../ = workspace root
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

#[test]
fn shipforge_receipt_works() {
    let tmp = setup_test_repo();
    let output = run_shipforge(&["receipt", "-p", "."], tmp.path());
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Change Receipt"));
}

#[test]
fn shipforge_log_works() {
    let tmp = setup_test_repo();
    let _ = run_shipforge(&["receipt", "-p", "."], tmp.path());

    let output = run_shipforge(&["log"], tmp.path());
    assert!(output.status.success());
}

#[test]
fn shipforge_blast_works() {
    let tmp = setup_test_repo();
    std::fs::write(tmp.path().join("lib.rs"), "pub fn foo() {}").unwrap();
    Command::new("git")
        .args(["add", "."])
        .current_dir(tmp.path())
        .output()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "add lib"])
        .current_dir(tmp.path())
        .output()
        .unwrap();
    let output = run_shipforge(&["blast", "lib.rs", "-p", "."], tmp.path());
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Blast Radius"));
}
