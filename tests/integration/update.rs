use assert_cmd::Command;
use predicates::prelude::*;

fn setup_instance(dir: &tempfile::TempDir, name: &str) {
    let inst_dir = dir.path().join("instances").join(name);
    std::fs::create_dir_all(&inst_dir).unwrap();

    let state = serde_json::json!({
        "name": name,
        "port": 18789,
        "provider": "anthropic",
        "model": "claude-sonnet-4-20250514",
        "image": "ghcr.io/openclaw/openclaw:latest",
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

    std::fs::write(
        inst_dir.join(".env"),
        "OPENCLAW_MODEL=claude-sonnet-4-20250514\nOPENCLAW_AUTH_TOKEN=tok123\n",
    )
    .unwrap();
}

#[test]
fn test_update_model() {
    let dir = tempfile::TempDir::new().unwrap();
    setup_instance(&dir, "updatemodel");

    Command::cargo_bin("moltctrl")
        .unwrap()
        .args(["update", "updatemodel", "--model", "gpt-4o"])
        .env("MOLTCTRL_DIR", dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Model updated to: gpt-4o"));

    // Verify instance.json
    let json =
        std::fs::read_to_string(dir.path().join("instances/updatemodel/instance.json")).unwrap();
    assert!(json.contains("gpt-4o"));

    // Verify .env
    let env = std::fs::read_to_string(dir.path().join("instances/updatemodel/.env")).unwrap();
    assert!(env.contains("OPENCLAW_MODEL=gpt-4o"));
}

#[test]
fn test_update_mem() {
    let dir = tempfile::TempDir::new().unwrap();
    setup_instance(&dir, "updatemem");

    Command::cargo_bin("moltctrl")
        .unwrap()
        .args(["update", "updatemem", "--mem", "4g"])
        .env("MOLTCTRL_DIR", dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Memory limit updated to: 4g"));

    let json =
        std::fs::read_to_string(dir.path().join("instances/updatemem/instance.json")).unwrap();
    assert!(json.contains("\"mem\": \"4g\""));
}

#[test]
fn test_update_no_changes() {
    let dir = tempfile::TempDir::new().unwrap();
    setup_instance(&dir, "updateno");

    Command::cargo_bin("moltctrl")
        .unwrap()
        .args(["update", "updateno"])
        .env("MOLTCTRL_DIR", dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("No updates specified"));
}

#[test]
fn test_update_nonexistent() {
    let dir = tempfile::TempDir::new().unwrap();
    std::fs::create_dir_all(dir.path().join("instances")).unwrap();

    Command::cargo_bin("moltctrl")
        .unwrap()
        .args(["update", "noexist", "--model", "test"])
        .env("MOLTCTRL_DIR", dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}
