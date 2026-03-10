#!/usr/bin/env bash
# Common test helpers for moltctrl bats tests

# Load bats libraries
BATS_TEST_HELPER_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
load "${BATS_TEST_HELPER_DIR}/bats-support/load"
load "${BATS_TEST_HELPER_DIR}/bats-assert/load"

# Path to the moltctrl script
MOLTCTRL_BIN="$(cd "${BATS_TEST_HELPER_DIR}/../.." && pwd)/moltctrl"

# Setup temp dir for each test and override MOLTCTRL_DIR
setup_common() {
    TEST_TEMP_DIR="$(mktemp -d)"
    export MOLTCTRL_DIR="${TEST_TEMP_DIR}/moltctrl_state"
    export NO_COLOR=1
    mkdir -p "${MOLTCTRL_DIR}/instances"
}

# Cleanup temp dir after each test
teardown_common() {
    if [[ -d "${TEST_TEMP_DIR:-}" ]]; then
        rm -rf "${TEST_TEMP_DIR}"
    fi
}

# Add mock docker to PATH
setup_mock_docker() {
    local mock_dir="${BATS_TEST_HELPER_DIR}/mocks"
    chmod +x "${mock_dir}/docker"
    export PATH="${mock_dir}:${PATH}"
}

# Source moltctrl for unit testing (functions become available)
source_moltctrl() {
    # shellcheck source=/dev/null
    source "${MOLTCTRL_BIN}"
}

# Create a minimal instance for testing
create_test_instance() {
    local name="${1:-testinst}"
    local port="${2:-18789}"
    local provider="${3:-anthropic}"
    local inst_dir="${MOLTCTRL_DIR}/instances/${name}"
    mkdir -p "$inst_dir"

    # Create instance.json
    cat > "${inst_dir}/instance.json" <<EOF
{
  "name": "${name}",
  "port": ${port},
  "provider": "${provider}",
  "model": "test-model",
  "image": "ghcr.io/openclaw/openclaw:latest",
  "created": "2025-01-01T00:00:00Z",
  "status": "running",
  "token": "deadbeef1234567890abcdef12345678",
  "mem": "2g",
  "cpus": "2",
  "pids": "256",
  "paired_keys": []
}
EOF

    # Create .env file
    cat > "${inst_dir}/.env" <<EOF
ANTHROPIC_API_KEY=sk-test-key
OPENCLAW_PROVIDER=${provider}
OPENCLAW_MODEL=test-model
OPENCLAW_SANDBOX_MODE=all
OPENCLAW_WORKSPACE_ACCESS=none
OPENCLAW_AUTH_TOKEN=deadbeef1234567890abcdef12345678
EOF
    chmod 600 "${inst_dir}/.env"

    # Create docker-compose.yml stub
    cat > "${inst_dir}/docker-compose.yml" <<EOF
version: "3.8"
services:
  openclaw:
    image: ghcr.io/openclaw/openclaw:latest
EOF
}
