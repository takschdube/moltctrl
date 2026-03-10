#!/usr/bin/env bats
# Integration tests for the create flow (mocked Docker)

load test_helper/common

setup() {
    setup_common
    setup_mock_docker
}

teardown() {
    teardown_common
}

@test "create: creates instance with required args" {
    run "${MOLTCTRL_BIN}" create testinst --provider anthropic --api-key sk-test-key-123
    assert_success
    assert_output --partial "Instance details:"
    assert_output --partial "testinst"
    assert_output --partial "anthropic"

    # Verify instance files exist
    assert [ -d "${MOLTCTRL_DIR}/instances/testinst" ]
    assert [ -f "${MOLTCTRL_DIR}/instances/testinst/instance.json" ]
    assert [ -f "${MOLTCTRL_DIR}/instances/testinst/.env" ]
    assert [ -f "${MOLTCTRL_DIR}/instances/testinst/docker-compose.yml" ]
}

@test "create: .env file has correct permissions" {
    run "${MOLTCTRL_BIN}" create sectest --provider anthropic --api-key sk-test-key-123
    assert_success

    local perms
    perms="$(stat -c '%a' "${MOLTCTRL_DIR}/instances/sectest/.env")"
    assert_equal "$perms" "600"
}

@test "create: .env contains provider key" {
    run "${MOLTCTRL_BIN}" create envtest --provider anthropic --api-key sk-test-key-123
    assert_success

    run cat "${MOLTCTRL_DIR}/instances/envtest/.env"
    assert_output --partial "ANTHROPIC_API_KEY=sk-test-key-123"
    assert_output --partial "OPENCLAW_PROVIDER=anthropic"
    assert_output --partial "OPENCLAW_SANDBOX_MODE=all"
    assert_output --partial "OPENCLAW_WORKSPACE_ACCESS=none"
}

@test "create: instance.json has correct values" {
    run "${MOLTCTRL_BIN}" create jsontest --provider openai --api-key sk-openai-123 --model gpt-4o-mini --port 18800
    assert_success

    local provider
    provider="$(jq -r '.provider' "${MOLTCTRL_DIR}/instances/jsontest/instance.json")"
    assert_equal "$provider" "openai"

    local port
    port="$(jq -r '.port' "${MOLTCTRL_DIR}/instances/jsontest/instance.json")"
    assert_equal "$port" "18800"

    local model
    model="$(jq -r '.model' "${MOLTCTRL_DIR}/instances/jsontest/instance.json")"
    assert_equal "$model" "gpt-4o-mini"
}

@test "create: fails without name" {
    run "${MOLTCTRL_BIN}" create
    assert_failure
    assert_output --partial "Usage: moltctrl create"
}

@test "create: fails with invalid name" {
    run "${MOLTCTRL_BIN}" create "1bad-name" --provider anthropic --api-key sk-test
    assert_failure
    assert_output --partial "Invalid instance name"
}

@test "create: fails on duplicate name" {
    run "${MOLTCTRL_BIN}" create duptest --provider anthropic --api-key sk-test-key-123
    assert_success

    run "${MOLTCTRL_BIN}" create duptest --provider anthropic --api-key sk-test-key-123
    assert_failure
    assert_output --partial "already exists"
}

@test "create: custom port is respected" {
    run "${MOLTCTRL_BIN}" create porttest --provider ollama --port 19000
    assert_success

    local port
    port="$(jq -r '.port' "${MOLTCTRL_DIR}/instances/porttest/instance.json")"
    assert_equal "$port" "19000"
}

@test "create: ollama needs no API key" {
    run "${MOLTCTRL_BIN}" create ollamatest --provider ollama
    assert_success
    assert_output --partial "ollama"
}

@test "create: docker-compose.yml has security hardening" {
    run "${MOLTCTRL_BIN}" create securitytest --provider anthropic --api-key sk-test-key-123
    assert_success

    local compose="${MOLTCTRL_DIR}/instances/securitytest/docker-compose.yml"
    assert [ -f "$compose" ]

    run cat "$compose"
    assert_output --partial 'read_only: true'
    assert_output --partial 'no-new-privileges:true'
    assert_output --partial '127.0.0.1:'
    assert_output --partial 'user: "1000:1000"'
}
