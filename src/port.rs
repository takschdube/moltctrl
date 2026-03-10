use std::net::TcpListener;

use anyhow::{bail, Result};

use crate::config;
use crate::state;

/// Check if a port is available using TcpListener::bind
pub fn is_port_available(port: u16) -> bool {
    TcpListener::bind(("127.0.0.1", port)).is_ok()
}

/// Check if a port is in use by any moltctrl instance
fn port_in_use_by_instance(port: u16) -> Result<bool> {
    let names = state::list_names()?;
    for name in &names {
        if let Ok(instance) = state::InstanceState::load(name) {
            if instance.port == port {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

/// Allocate the first available port in the moltctrl range
pub fn allocate_port() -> Result<u16> {
    for port in config::PORT_MIN..=config::PORT_MAX {
        if !port_in_use_by_instance(port)? && is_port_available(port) {
            return Ok(port);
        }
    }
    bail!(
        "No available ports in range {}-{}",
        config::PORT_MIN,
        config::PORT_MAX
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_port_available() {
        // Bind a port, then check it's unavailable
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        assert!(!is_port_available(port));
        drop(listener);
        assert!(is_port_available(port));
    }
}
