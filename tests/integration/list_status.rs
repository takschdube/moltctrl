use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_list_empty() {
    let dir = tempfile::TempDir::new().unwrap();
    Command::cargo_bin("moltctrl")
        .unwrap()
        .args(["list"])
        .env("MOLTCTRL_DIR", dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("No instances found"));
}

#[test]
fn test_list_with_instance() {
    let dir = tempfile::TempDir::new().unwrap();
    let inst_dir = dir.path().join("instances").join("testinst");
    std::fs::create_dir_all(&inst_dir).unwrap();

    let state = serde_json::json!({
        "name": "testinst",
        "port": 18789,
        "provider": "anthropic",
        "model": "claude-sonnet-4-20250514",
        "image": "ghcr.io/openclaw/openclaw:latest",
        "created": "2024-01-15T10:30:00Z",
        "status": "running",
        "token": "abc123",
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
        .args(["list"])
        .env("MOLTCTRL_DIR", dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("testinst"))
        .stdout(predicate::str::contains("anthropic"))
        .stdout(predicate::str::contains("18789"));
}

#[test]
fn test_status_not_found() {
    let dir = tempfile::TempDir::new().unwrap();
    std::fs::create_dir_all(dir.path().join("instances")).unwrap();
    Command::cargo_bin("moltctrl")
        .unwrap()
        .args(["status", "nonexistent"])
        .env("MOLTCTRL_DIR", dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_status_existing() {
    let dir = tempfile::TempDir::new().unwrap();
    let inst_dir = dir.path().join("instances").join("myinst");
    std::fs::create_dir_all(&inst_dir).unwrap();

    let state = serde_json::json!({
        "name": "myinst",
        "port": 18790,
        "provider": "openai",
        "model": "gpt-4o",
        "image": "test:latest",
        "created": "2024-02-01T00:00:00Z",
        "status": "stopped",
        "token": "tok123",
        "mem": "4g",
        "cpus": "4",
        "pids": "512",
        "paired_keys": []
    });

    std::fs::write(
        inst_dir.join("instance.json"),
        serde_json::to_string_pretty(&state).unwrap(),
    )
    .unwrap();

    Command::cargo_bin("moltctrl")
        .unwrap()
        .args(["status", "myinst"])
        .env("MOLTCTRL_DIR", dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Instance: myinst"))
        .stdout(predicate::str::contains("Provider: openai"))
        .stdout(predicate::str::contains("Model:    gpt-4o"))
        .stdout(predicate::str::contains("Port:     18790"))
        .stdout(predicate::str::contains("Mem:      4g"));
}
