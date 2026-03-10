#!/usr/bin/env bats
# Unit tests for validation, color, and JSON helper functions

load test_helper/common

setup() {
    setup_common
    source_moltctrl
}

teardown() {
    teardown_common
}

# --- validate_name ---

@test "validate_name: accepts valid names" {
    validate_name "myagent"
    validate_name "my-agent"
    validate_name "my_agent"
    validate_name "Agent1"
    validate_name "a"
}

@test "validate_name: rejects empty name" {
    run validate_name ""
    assert_failure
    assert_output --partial "Instance name is required"
}

@test "validate_name: rejects names starting with number" {
    run validate_name "1agent"
    assert_failure
    assert_output --partial "Invalid instance name"
}

@test "validate_name: rejects names starting with hyphen" {
    run validate_name "-agent"
    assert_failure
    assert_output --partial "Invalid instance name"
}

@test "validate_name: rejects names with special chars" {
    run validate_name "my agent"
    assert_failure
    assert_output --partial "Invalid instance name"

    run validate_name "my.agent"
    assert_failure
    assert_output --partial "Invalid instance name"
}

@test "validate_name: rejects names over 63 chars" {
    local long_name
    long_name="a$(printf '%0.s1' {1..63})"
    run validate_name "$long_name"
    assert_failure
    assert_output --partial "Invalid instance name"
}

# --- validate_port ---

@test "validate_port: accepts valid ports" {
    validate_port "1024"
    validate_port "18789"
    validate_port "65535"
}

@test "validate_port: rejects low ports" {
    run validate_port "80"
    assert_failure
    assert_output --partial "Invalid port"
}

@test "validate_port: rejects high ports" {
    run validate_port "65536"
    assert_failure
    assert_output --partial "Invalid port"
}

@test "validate_port: rejects non-numeric" {
    run validate_port "abc"
    assert_failure
    assert_output --partial "Invalid port"
}

# --- validate_provider ---

@test "validate_provider: accepts all valid providers" {
    validate_provider "anthropic"
    validate_provider "openai"
    validate_provider "google"
    validate_provider "aws-bedrock"
    validate_provider "openrouter"
    validate_provider "ollama"
}

@test "validate_provider: rejects unknown provider" {
    run validate_provider "unknown"
    assert_failure
    assert_output --partial "Unknown provider"
}

# --- info/warn/error/success output ---

@test "info: prints info message" {
    run info "test message"
    assert_success
    assert_output --partial "test message"
}

@test "warn: prints warning to stderr" {
    run warn "test warning"
    assert_success
    assert_output --partial "test warning"
}

@test "error: prints error to stderr" {
    run error "test error"
    assert_success
    assert_output --partial "test error"
}

@test "die: prints error and exits non-zero" {
    run die "fatal error"
    assert_failure
    assert_output --partial "fatal error"
}

# --- JSON state ---

@test "instance_create_json: creates valid JSON" {
    local inst_dir="${MOLTCTRL_DIR}/instances/testjson"
    mkdir -p "$inst_dir"
    instance_create_json "testjson" "18789" "anthropic" "claude-3" "test:latest"

    assert [ -f "${inst_dir}/instance.json" ]

    local name
    name="$(jq -r '.name' "${inst_dir}/instance.json")"
    assert_equal "$name" "testjson"

    local port
    port="$(jq -r '.port' "${inst_dir}/instance.json")"
    assert_equal "$port" "18789"
}

@test "instance_get: retrieves values" {
    create_test_instance "gettest"
    local result
    result="$(instance_get "gettest" "provider")"
    assert_equal "$result" "anthropic"
}

@test "instance_set: updates values" {
    create_test_instance "settest"
    instance_set "settest" "status" "stopped"
    local result
    result="$(instance_get "settest" "status")"
    assert_equal "$result" "stopped"
}

@test "instance_list_names: lists instance names" {
    create_test_instance "inst1"
    create_test_instance "inst2"
    run instance_list_names
    assert_success
    assert_output --partial "inst1"
    assert_output --partial "inst2"
}
