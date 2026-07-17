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

fn run_devpulse(args: &[&str], project_dir: &std::path::Path) -> std::process::Output {
    let manifest = format!("{}/Cargo.toml", workspace_root());
    let mut cmd_args = vec!["run", "--manifest-path", &manifest, "-p", "devpulse", "--"];
    cmd_args.extend_from_slice(args);
    Command::new("cargo")
        .args(&cmd_args)
        .current_dir(project_dir)
        .output()
        .unwrap()
}

#[test]
fn devpulse_event_works() {
    let tmp = setup_test_repo();
    let output = run_devpulse(&["event", "-k", "coding", "-m", "30", "-d", "test session", "-p", "."], tmp.path());
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Recorded"));
}

#[test]
fn devpulse_chart_works() {
    let tmp = setup_test_repo();
    // Record some events first
    let _ = run_devpulse(&["event", "-k", "coding", "-m", "60", "-d", "morning", "-p", "."], tmp.path());

    let output = run_devpulse(&["chart", "--period", "day", "-p", "."], tmp.path());
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("DevPulse Chart"));
}

#[test]
fn devpulse_report_works() {
    let tmp = setup_test_repo();
    let _ = run_devpulse(&["event", "-k", "coding", "-m", "120", "-d", "feature work", "-p", "."], tmp.path());
    let _ = run_devpulse(&["event", "-k", "review", "-m", "30", "-d", "PR review", "-p", "."], tmp.path());

    let output = run_devpulse(&["report", "--period", "day", "-p", "."], tmp.path());
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("DevPulse Report"));
}
