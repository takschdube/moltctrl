#!/usr/bin/env bats
# Unit tests for template rendering

load test_helper/common

setup() {
    setup_common
    source_moltctrl
}

teardown() {
    teardown_common
}

@test "render_template: substitutes MOLTCTRL variables" {
    # Create a test template
    local tmpl="${TEST_TEMP_DIR}/test.yml.tmpl"
    local output="${TEST_TEMP_DIR}/test.yml"
    cat > "$tmpl" <<'EOF'
image: ${MOLTCTRL_IMAGE}
container_name: ${MOLTCTRL_CONTAINER_NAME}
ports:
  - "127.0.0.1:${MOLTCTRL_PORT}:18789"
mem_limit: ${MOLTCTRL_MEM_LIMIT}
cpus: ${MOLTCTRL_CPUS}
EOF

    export MOLTCTRL_IMAGE="test:latest"
    export MOLTCTRL_CONTAINER_NAME="moltctrl-test"
    export MOLTCTRL_PORT="18800"
    export MOLTCTRL_MEM_LIMIT="4g"
    export MOLTCTRL_CPUS="4"
    export MOLTCTRL_MEMSWAP_LIMIT="4g"
    export MOLTCTRL_PIDS_LIMIT="512"
    export MOLTCTRL_ENV_FILE="/tmp/test.env"
    export MOLTCTRL_VOLUME_PREFIX="moltctrl_test"

    render_template "$tmpl" "$output"

    assert [ -f "$output" ]

    run cat "$output"
    assert_output --partial "image: test:latest"
    assert_output --partial "container_name: moltctrl-test"
    assert_output --partial "127.0.0.1:18800:18789"
    assert_output --partial "mem_limit: 4g"
    assert_output --partial "cpus: 4"
}

@test "render_template: does not substitute non-MOLTCTRL vars" {
    local tmpl="${TEST_TEMP_DIR}/safe.yml.tmpl"
    local output="${TEST_TEMP_DIR}/safe.yml"
    cat > "$tmpl" <<'EOF'
home: ${HOME}
user: ${USER}
image: ${MOLTCTRL_IMAGE}
EOF

    export MOLTCTRL_IMAGE="test:latest"
    export MOLTCTRL_CONTAINER_NAME=""
    export MOLTCTRL_PORT=""
    export MOLTCTRL_MEM_LIMIT=""
    export MOLTCTRL_MEMSWAP_LIMIT=""
    export MOLTCTRL_CPUS=""
    export MOLTCTRL_PIDS_LIMIT=""
    export MOLTCTRL_ENV_FILE=""
    export MOLTCTRL_VOLUME_PREFIX=""

    render_template "$tmpl" "$output"

    run cat "$output"
    # HOME and USER should NOT be substituted
    assert_output --partial '${HOME}'
    assert_output --partial '${USER}'
    # MOLTCTRL_IMAGE should be substituted
    assert_output --partial "image: test:latest"
}

@test "render_template: fails on missing template" {
    run render_template "/nonexistent/template.yml" "/tmp/output.yml"
    assert_failure
    assert_output --partial "Template not found"
}
