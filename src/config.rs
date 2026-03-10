use std::path::PathBuf;

use anyhow::Result;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const DEFAULT_IMAGE: &str = "ghcr.io/openclaw/openclaw:latest";
pub const PORT_MIN: u16 = 18789;
pub const PORT_MAX: u16 = 18889;
pub const DEFAULT_MEM: &str = "2g";
pub const DEFAULT_CPUS: &str = "2";
pub const DEFAULT_PIDS: &str = "256";
pub const COMPOSE_PREFIX: &str = "moltctrl";

/// Returns the base moltctrl directory (~/.moltctrl or MOLTCTRL_DIR env)
pub fn moltctrl_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("MOLTCTRL_DIR") {
        PathBuf::from(dir)
    } else {
        dirs::home_dir()
            .expect("Could not determine home directory")
            .join(".moltctrl")
    }
}

/// Returns the instances directory
pub fn instances_dir() -> PathBuf {
    moltctrl_dir().join("instances")
}

/// Returns the instance directory for a given name
pub fn instance_dir(name: &str) -> PathBuf {
    instances_dir().join(name)
}

/// Ensure all required directories exist
pub fn ensure_dirs() -> Result<()> {
    let dirs = [moltctrl_dir(), instances_dir()];
    for dir in &dirs {
        std::fs::create_dir_all(dir)?;
    }
    Ok(())
}
