use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_doctor_output() {
    Command::cargo_bin("moltctrl")
        .unwrap()
        .args(["doctor"])
        .assert()
        .stdout(predicate::str::contains("moltctrl doctor"))
        .stdout(predicate::str::contains("Docker:"))
        .stdout(predicate::str::contains("Docker Compose:"))
        .stdout(predicate::str::contains("not needed"));
}

#[test]
fn test_doctor_shows_builtin_deps() {
    Command::cargo_bin("moltctrl")
        .unwrap()
        .args(["doctor"])
        .assert()
        .stdout(predicate::str::contains("jq:"))
        .stdout(predicate::str::contains("not needed (built-in JSON)"))
        .stdout(predicate::str::contains("envsubst:"))
        .stdout(predicate::str::contains("not needed (built-in templates)"))
        .stdout(predicate::str::contains("openssl:"))
        .stdout(predicate::str::contains("not needed (built-in token gen)"))
        .stdout(predicate::str::contains("websocat:"))
        .stdout(predicate::str::contains("not needed (built-in WebSocket)"));
}
