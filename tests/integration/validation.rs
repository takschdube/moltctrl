use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_create_invalid_name_starts_with_number() {
    Command::cargo_bin("moltctrl")
        .unwrap()
        .args([
            "create",
            "1invalid",
            "--provider",
            "anthropic",
            "--api-key",
            "test",
        ])
        .env("MOLTCTRL_DIR", "/tmp/moltctrl-test-validation")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid instance name"));
}

#[test]
fn test_create_invalid_name_starts_with_hyphen() {
    Command::cargo_bin("moltctrl")
        .unwrap()
        .args([
            "create",
            "-invalid",
            "--provider",
            "anthropic",
            "--api-key",
            "test",
        ])
        .env("MOLTCTRL_DIR", "/tmp/moltctrl-test-validation")
        .assert()
        .failure();
}

#[test]
fn test_create_invalid_provider() {
    Command::cargo_bin("moltctrl")
        .unwrap()
        .args([
            "create",
            "test",
            "--provider",
            "badprovider",
            "--api-key",
            "test",
        ])
        .env("MOLTCTRL_DIR", "/tmp/moltctrl-test-validation")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unknown provider"));
}

#[test]
fn test_create_no_name() {
    Command::cargo_bin("moltctrl")
        .unwrap()
        .args(["create"])
        .env("MOLTCTRL_DIR", "/tmp/moltctrl-test-validation")
        .assert()
        .failure();
}

#[test]
fn test_unknown_command() {
    Command::cargo_bin("moltctrl")
        .unwrap()
        .args(["nonexistent"])
        .assert()
        .failure();
}

#[test]
fn test_version_output() {
    Command::cargo_bin("moltctrl")
        .unwrap()
        .args(["version"])
        .assert()
        .success()
        .stdout(predicate::str::contains("moltctrl v"));
}

#[test]
fn test_help_output() {
    Command::cargo_bin("moltctrl")
        .unwrap()
        .args(["--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Security-hardened OpenClaw AI agent instance manager",
        ));
}

#[test]
fn test_valid_providers_accepted() {
    for provider in &["anthropic", "openai", "google", "aws-bedrock", "openrouter"] {
        // Use --process mode so it doesn't try to pull Docker images
        // It will create the instance and exit cleanly
        let tmp = format!("/tmp/moltctrl-test-prov-{}", provider);
        let result = Command::cargo_bin("moltctrl")
            .unwrap()
            .args([
                "create",
                "testinst",
                "--provider",
                provider,
                "--api-key",
                "test-key-123",
                "--process",
            ])
            .env("MOLTCTRL_DIR", &tmp)
            .timeout(std::time::Duration::from_secs(10))
            .assert();
        // Should not fail with "Unknown provider"
        result.stderr(predicate::str::contains("Unknown provider").not());
        // Clean up
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
