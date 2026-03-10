#!/usr/bin/env bats
# Unit tests for provider resolution

load test_helper/common

setup() {
    setup_common
    source_moltctrl
}

teardown() {
    teardown_common
}

@test "provider_env_key: returns correct env var for anthropic" {
    local result
    result="$(provider_env_key "anthropic")"
    assert_equal "$result" "ANTHROPIC_API_KEY"
}

@test "provider_env_key: returns correct env var for openai" {
    local result
    result="$(provider_env_key "openai")"
    assert_equal "$result" "OPENAI_API_KEY"
}

@test "provider_env_key: returns empty for ollama" {
    local result
    result="$(provider_env_key "ollama")"
    assert_equal "$result" ""
}

@test "provider_default_model: returns correct model for anthropic" {
    local result
    result="$(provider_default_model "anthropic")"
    assert_equal "$result" "claude-sonnet-4-20250514"
}

@test "provider_default_model: returns correct model for ollama" {
    local result
    result="$(provider_default_model "ollama")"
    assert_equal "$result" "llama3.1"
}

@test "resolve_provider: uses flag provider with key" {
    local result
    result="$(resolve_provider "anthropic" "sk-test-key")"
    assert_equal "$result" "anthropic:sk-test-key"
}

@test "resolve_provider: uses MOLTCTRL_PROVIDER env var" {
    export MOLTCTRL_PROVIDER="openai"
    local result
    result="$(resolve_provider "" "sk-openai-key")"
    assert_equal "$result" "openai:sk-openai-key"
    unset MOLTCTRL_PROVIDER
}

@test "resolve_provider: ollama needs no API key" {
    local result
    result="$(resolve_provider "ollama" "")"
    assert_equal "$result" "ollama:"
}

@test "resolve_provider: uses env var for API key" {
    export ANTHROPIC_API_KEY="sk-from-env"
    local result
    result="$(resolve_provider "anthropic" "")"
    assert_equal "$result" "anthropic:sk-from-env"
    unset ANTHROPIC_API_KEY
}

@test "resolve_provider: fails without provider in non-interactive mode" {
    run resolve_provider "" "" < /dev/null
    assert_failure
    assert_output --partial "No provider specified"
}

@test "resolve_provider: rejects invalid provider" {
    run resolve_provider "invalid" "key"
    assert_failure
    assert_output --partial "Unknown provider"
}
