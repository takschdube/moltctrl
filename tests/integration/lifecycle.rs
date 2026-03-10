use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_start_nonexistent() {
    let dir = tempfile::TempDir::new().unwrap();
    std::fs::create_dir_all(dir.path().join("instances")).unwrap();

    Command::cargo_bin("moltctrl")
        .unwrap()
        .args(["start", "nonexistent"])
        .env("MOLTCTRL_DIR", dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_stop_nonexistent() {
    let dir = tempfile::TempDir::new().unwrap();
    std::fs::create_dir_all(dir.path().join("instances")).unwrap();

    Command::cargo_bin("moltctrl")
        .unwrap()
        .args(["stop", "nonexistent"])
        .env("MOLTCTRL_DIR", dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_restart_nonexistent() {
    let dir = tempfile::TempDir::new().unwrap();
    std::fs::create_dir_all(dir.path().join("instances")).unwrap();

    Command::cargo_bin("moltctrl")
        .unwrap()
        .args(["restart", "nonexistent"])
        .env("MOLTCTRL_DIR", dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_logs_nonexistent() {
    let dir = tempfile::TempDir::new().unwrap();
    std::fs::create_dir_all(dir.path().join("instances")).unwrap();

    Command::cargo_bin("moltctrl")
        .unwrap()
        .args(["logs", "nonexistent"])
        .env("MOLTCTRL_DIR", dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}
