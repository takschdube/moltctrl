use assert_cmd::Command;
use predicates::prelude::*;

fn setup_instance(dir: &tempfile::TempDir, name: &str) {
    let inst_dir = dir.path().join("instances").join(name);
    std::fs::create_dir_all(&inst_dir).unwrap();

    let state = serde_json::json!({
        "name": name,
        "port": 18789,
        "provider": "anthropic",
        "model": "test",
        "image": "test:latest",
        "created": "2024-01-01T00:00:00Z",
        "status": "running",
        "token": "tok123",
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
}

#[test]
fn test_pair_approve() {
    let dir = tempfile::TempDir::new().unwrap();
    setup_instance(&dir, "pairtest");

    Command::cargo_bin("moltctrl")
        .unwrap()
        .args(["pair", "approve", "pairtest", "--label", "mykey"])
        .env("MOLTCTRL_DIR", dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Pairing key created"))
        .stdout(predicate::str::contains("Label: mykey"));
}

#[test]
fn test_pair_list_empty() {
    let dir = tempfile::TempDir::new().unwrap();
    setup_instance(&dir, "pairlist");

    Command::cargo_bin("moltctrl")
        .unwrap()
        .args(["pair", "list", "pairlist"])
        .env("MOLTCTRL_DIR", dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("No pairing keys"));
}

#[test]
fn test_pair_approve_then_list() {
    let dir = tempfile::TempDir::new().unwrap();
    setup_instance(&dir, "pairflow");

    // Approve a key
    Command::cargo_bin("moltctrl")
        .unwrap()
        .args(["pair", "approve", "pairflow", "--label", "testkey"])
        .env("MOLTCTRL_DIR", dir.path())
        .assert()
        .success();

    // List should show the key
    Command::cargo_bin("moltctrl")
        .unwrap()
        .args(["pair", "list", "pairflow"])
        .env("MOLTCTRL_DIR", dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("testkey"));
}

#[test]
fn test_pair_revoke() {
    let dir = tempfile::TempDir::new().unwrap();
    setup_instance(&dir, "pairrevoke");

    // Approve
    Command::cargo_bin("moltctrl")
        .unwrap()
        .args(["pair", "approve", "pairrevoke", "--label", "revokekey"])
        .env("MOLTCTRL_DIR", dir.path())
        .assert()
        .success();

    // Revoke
    Command::cargo_bin("moltctrl")
        .unwrap()
        .args(["pair", "revoke", "pairrevoke", "--label", "revokekey"])
        .env("MOLTCTRL_DIR", dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("revoked"));

    // Verify it's gone
    Command::cargo_bin("moltctrl")
        .unwrap()
        .args(["pair", "list", "pairrevoke"])
        .env("MOLTCTRL_DIR", dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("No pairing keys"));
}

#[test]
fn test_pair_revoke_nonexistent() {
    let dir = tempfile::TempDir::new().unwrap();
    setup_instance(&dir, "pairnokey");

    Command::cargo_bin("moltctrl")
        .unwrap()
        .args(["pair", "revoke", "pairnokey", "--label", "doesnotexist"])
        .env("MOLTCTRL_DIR", dir.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("No pairing key with label"));
}
