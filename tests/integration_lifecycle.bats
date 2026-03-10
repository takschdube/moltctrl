#!/usr/bin/env bats
# Integration tests for list/destroy/start/stop/status commands

load test_helper/common

setup() {
    setup_common
    setup_mock_docker
}

teardown() {
    teardown_common
}

# --- list ---

@test "list: shows empty message when no instances" {
    run "${MOLTCTRL_BIN}" list
    assert_success
    assert_output --partial "No instances found"
}

@test "list: shows instances" {
    create_test_instance "inst1" 18789 "anthropic"
    create_test_instance "inst2" 18790 "openai"

    run "${MOLTCTRL_BIN}" list
    assert_success
    assert_output --partial "inst1"
    assert_output --partial "inst2"
    assert_output --partial "anthropic"
    assert_output --partial "openai"
}

@test "list: alias 'ls' works" {
    create_test_instance "lsalias"
    run "${MOLTCTRL_BIN}" ls
    assert_success
    assert_output --partial "lsalias"
}

# --- status ---

@test "status: shows instance details" {
    create_test_instance "statusinst"

    run "${MOLTCTRL_BIN}" status statusinst
    assert_success
    assert_output --partial "Instance: statusinst"
    assert_output --partial "Provider: anthropic"
    assert_output --partial "Port:     18789"
}

@test "status: fails for non-existent instance" {
    run "${MOLTCTRL_BIN}" status nonexistent
    assert_failure
    assert_output --partial "not found"
}

@test "status: fails without name" {
    run "${MOLTCTRL_BIN}" status
    assert_failure
    assert_output --partial "Usage"
}

# --- start ---

@test "start: starts a stopped instance" {
    create_test_instance "startinst"

    run "${MOLTCTRL_BIN}" start startinst
    assert_success
    assert_output --partial "started"
}

@test "start: fails for non-existent instance" {
    run "${MOLTCTRL_BIN}" start nonexistent
    assert_failure
    assert_output --partial "not found"
}

# --- stop ---

@test "stop: stops a running instance" {
    create_test_instance "stopinst"

    run "${MOLTCTRL_BIN}" stop stopinst
    assert_success
    assert_output --partial "stopped"
}

# --- restart ---

@test "restart: restarts an instance" {
    create_test_instance "restartinst"

    run "${MOLTCTRL_BIN}" restart restartinst
    assert_success
    assert_output --partial "restarted"
}

# --- destroy ---

@test "destroy: removes instance with --force" {
    create_test_instance "destroyinst"
    assert [ -d "${MOLTCTRL_DIR}/instances/destroyinst" ]

    run "${MOLTCTRL_BIN}" destroy destroyinst --force
    assert_success
    assert_output --partial "destroyed"
    assert [ ! -d "${MOLTCTRL_DIR}/instances/destroyinst" ]
}

@test "destroy: global --force flag works" {
    create_test_instance "destroyinst2"

    run "${MOLTCTRL_BIN}" --force destroy destroyinst2
    assert_success
    assert_output --partial "destroyed"
}

@test "destroy: fails for non-existent instance" {
    run "${MOLTCTRL_BIN}" destroy nonexistent --force
    assert_failure
    assert_output --partial "not found"
}

@test "destroy: fails without name" {
    run "${MOLTCTRL_BIN}" destroy
    assert_failure
    assert_output --partial "Usage"
}

# --- logs ---

@test "logs: shows logs" {
    create_test_instance "logsinst"

    run "${MOLTCTRL_BIN}" logs logsinst
    assert_success
}

@test "logs: fails for non-existent instance" {
    run "${MOLTCTRL_BIN}" logs nonexistent
    assert_failure
    assert_output --partial "not found"
}

# --- token ---

@test "token: shows current token" {
    create_test_instance "tokeninst"

    run "${MOLTCTRL_BIN}" token tokeninst
    assert_success
    assert_output "deadbeef1234567890abcdef12345678"
}

@test "token: regenerates token" {
    create_test_instance "regeninst"

    run "${MOLTCTRL_BIN}" token regeninst --regenerate
    assert_success
    assert_output --partial "Token regenerated"
    # The new token should be different from the original
    local new_token
    new_token="$(jq -r '.token' "${MOLTCTRL_DIR}/instances/regeninst/instance.json")"
    assert [ "$new_token" != "deadbeef1234567890abcdef12345678" ]
}

# --- version ---

@test "version: shows version" {
    run "${MOLTCTRL_BIN}" version
    assert_success
    assert_output --partial "moltctrl v"
}

# --- help ---

@test "help: shows help text" {
    run "${MOLTCTRL_BIN}" help
    assert_success
    assert_output --partial "Usage: moltctrl"
    assert_output --partial "Commands:"
}

@test "help: shows command-specific help" {
    run "${MOLTCTRL_BIN}" help create
    assert_success
    assert_output --partial "moltctrl create"
    assert_output --partial "--provider"
}

# --- unknown command ---

@test "unknown command: fails with error" {
    run "${MOLTCTRL_BIN}" foobar
    assert_failure
    assert_output --partial "Unknown command"
}

# --- global flags ---

@test "global: --no-color works" {
    run "${MOLTCTRL_BIN}" --no-color help
    assert_success
}

@test "global: --verbose works" {
    create_test_instance "verboseinst"
    run "${MOLTCTRL_BIN}" --verbose status verboseinst
    assert_success
}
