use moltctrl::port;

#[test]
fn test_port_availability_check() {
    // A high random port should be available
    let port = 59123;
    let result = port::is_port_available(port);
    // We can't guarantee it's available, but the function should not panic
    let _ = result;
}

#[test]
fn test_port_bind_check() {
    // Bind a port, verify it shows as unavailable
    let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    assert!(!port::is_port_available(port));
    drop(listener);
    assert!(port::is_port_available(port));
}
