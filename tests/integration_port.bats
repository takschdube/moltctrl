#!/usr/bin/env bats
# Integration tests for port allocation logic

load test_helper/common

setup() {
    setup_common
    source_moltctrl
}

teardown() {
    teardown_common
}

@test "allocate_port: returns first available port when no instances" {
    local port
    port="$(allocate_port)"
    # Port should be in valid range (might not be 18789 if that port is in use on this host)
    assert [ "$port" -ge "$MOLTCTRL_PORT_MIN" ]
    assert [ "$port" -le "$MOLTCTRL_PORT_MAX" ]
}

@test "allocate_port: skips ports used by existing instances" {
    create_test_instance "inst1" 18789
    local port
    port="$(allocate_port)"
    assert_equal "$port" "18790"
}

@test "allocate_port: skips multiple used ports" {
    create_test_instance "inst1" 18789
    create_test_instance "inst2" 18790
    create_test_instance "inst3" 18791
    local port
    port="$(allocate_port)"
    assert_equal "$port" "18792"
}

@test "port_in_use_by_instance: detects used port" {
    create_test_instance "inst1" 18789
    run port_in_use_by_instance 18789
    assert_success
}

@test "port_in_use_by_instance: detects free port" {
    create_test_instance "inst1" 18789
    run port_in_use_by_instance 18790
    assert_failure
}

@test "allocate_port: handles non-contiguous allocations" {
    create_test_instance "inst1" 18789
    # inst2 skips to 18795, so 18790 should be next available
    create_test_instance "inst2" 18795
    local port
    port="$(allocate_port)"
    assert_equal "$port" "18790"
}
