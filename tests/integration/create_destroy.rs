use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_create_duplicate_instance() {
    let dir = tempfile::TempDir::new().unwrap();
    let inst_dir = dir.path().join("instances").join("existing");
    std::fs::create_dir_all(&inst_dir).unwrap();

    let state = serde_json::json!({
        "name": "existing",
        "port": 18789,
        "provider": "anthropic",
        "model": "claude-sonnet-4-20250514",
        "image": "test:latest",
        "created": "2024-01-01T00:00:00Z",
        "status": "running",
        "token": "tok",
        "mem": "2g",
        "cpus": "2",
        "pids": "256",
        "paired_keys": []
    });

    std::fs::write(
        inst_dir.join("instance.json"),
        serde_json::to_string_pretty(&state).unwrap(),
    )
    .unwrap();

    Command::cargo_bin("moltctrl")
        .unwrap()
        .args([
            "create",
            "existing",
            "--provider",
            "anthropic",
            "--api-key",
            "test",
        ])
        .env("MOLTCTRL_DIR", dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));
}

#[test]
fn test_destroy_nonexistent() {
    let dir = tempfile::TempDir::new().unwrap();
    std::fs::create_dir_all(dir.path().join("instances")).unwrap();

    Command::cargo_bin("moltctrl")
        .unwrap()
        .args(["destroy", "nonexistent", "--force"])
        .env("MOLTCTRL_DIR", dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_destroy_with_force() {
    let dir = tempfile::TempDir::new().unwrap();
    let inst_dir = dir.path().join("instances").join("todelete");
    std::fs::create_dir_all(&inst_dir).unwrap();

    let state = serde_json::json!({
        "name": "todelete",
        "port": 18789,
        "provider": "anthropic",
        "model": "test",
        "image": "test:latest",
        "created": "2024-01-01T00:00:00Z",
        "status": "stopped",
        "token": "tok",
        "mem": "2g",
        "cpus": "2",
        "pids": "256",
        "paired_keys": []
    });

    std::fs::write(
        inst_dir.join("instance.json"),
        serde_json::to_string_pretty(&state).unwrap(),
    )
    .unwrap();

    Command::cargo_bin("moltctrl")
        .unwrap()
        .args(["destroy", "todelete", "--force"])
        .env("MOLTCTRL_DIR", dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("destroyed"));

    // Verify instance directory is gone
    assert!(!inst_dir.exists());
}
