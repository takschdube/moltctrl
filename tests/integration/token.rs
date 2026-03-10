use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_token_show() {
    let dir = tempfile::TempDir::new().unwrap();
    let inst_dir = dir.path().join("instances").join("tokentest");
    std::fs::create_dir_all(&inst_dir).unwrap();

    let state = serde_json::json!({
        "name": "tokentest",
        "port": 18789,
        "provider": "anthropic",
        "model": "test",
        "image": "test:latest",
        "created": "2024-01-01T00:00:00Z",
        "status": "running",
        "token": "abc123def456",
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
        .args(["token", "tokentest"])
        .env("MOLTCTRL_DIR", dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("abc123def456"));
}

#[test]
fn test_token_regenerate() {
    let dir = tempfile::TempDir::new().unwrap();
    let inst_dir = dir.path().join("instances").join("regentest");
    std::fs::create_dir_all(&inst_dir).unwrap();

    let state = serde_json::json!({
        "name": "regentest",
        "port": 18789,
        "provider": "anthropic",
        "model": "test",
        "image": "test:latest",
        "created": "2024-01-01T00:00:00Z",
        "status": "running",
        "token": "oldtoken123",
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

    // Also write .env file for token update
    std::fs::write(inst_dir.join(".env"), "OPENCLAW_AUTH_TOKEN=oldtoken123\n").unwrap();

    let output = Command::cargo_bin("moltctrl")
        .unwrap()
        .args(["token", "regentest", "--regenerate"])
        .env("MOLTCTRL_DIR", dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("oldtoken123").not());

    // Verify the new token is 64 hex chars
    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    let new_token = stdout.lines().last().unwrap().trim();
    assert_eq!(new_token.len(), 64);
    assert!(new_token.chars().all(|c| c.is_ascii_hexdigit()));

    // Verify instance.json was updated
    let json_content = std::fs::read_to_string(inst_dir.join("instance.json")).unwrap();
    assert!(json_content.contains(new_token));

    // Verify .env was updated
    let env_content = std::fs::read_to_string(inst_dir.join(".env")).unwrap();
    assert!(env_content.contains(new_token));
}

#[test]
fn test_token_not_found() {
    let dir = tempfile::TempDir::new().unwrap();
    std::fs::create_dir_all(dir.path().join("instances")).unwrap();

    Command::cargo_bin("moltctrl")
        .unwrap()
        .args(["token", "nonexistent"])
        .env("MOLTCTRL_DIR", dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}
