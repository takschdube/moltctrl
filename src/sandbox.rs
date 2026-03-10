use std::path::PathBuf;

use anyhow::Result;

/// Configuration for a process sandbox
#[derive(Debug, Clone)]
pub struct SandboxConfig {
    pub working_dir: PathBuf,
    pub mem_limit_bytes: Option<u64>,
    pub cpu_limit: Option<u32>,
    pub pid_limit: Option<u32>,
}

/// Trait for platform-specific process sandboxing
pub trait ProcessSandbox {
    /// Apply sandbox restrictions to the current process (called pre-exec)
    fn apply(&self, config: &SandboxConfig) -> Result<()>;

    /// Clean up sandbox resources
    fn cleanup(&self) -> Result<()>;
}

/// Create a platform-specific sandbox implementation
pub fn create_sandbox() -> Box<dyn ProcessSandbox> {
    #[cfg(unix)]
    {
        Box::new(super::sandbox_unix::UnixSandbox::new())
    }
    #[cfg(windows)]
    {
        Box::new(super::sandbox_windows::WindowsSandbox::new())
    }
    #[cfg(not(any(unix, windows)))]
    {
        compile_error!("Unsupported platform for process sandboxing")
    }
}

/// Parse a memory limit string (e.g., "2g", "512m") into bytes
pub fn parse_mem_limit(s: &str) -> Option<u64> {
    let s = s.trim().to_lowercase();
    if let Some(num) = s.strip_suffix('g') {
        num.parse::<u64>().ok().map(|n| n * 1024 * 1024 * 1024)
    } else if let Some(num) = s.strip_suffix('m') {
        num.parse::<u64>().ok().map(|n| n * 1024 * 1024)
    } else if let Some(num) = s.strip_suffix('k') {
        num.parse::<u64>().ok().map(|n| n * 1024)
    } else {
        s.parse::<u64>().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_mem_limit() {
        assert_eq!(parse_mem_limit("2g"), Some(2 * 1024 * 1024 * 1024));
        assert_eq!(parse_mem_limit("512m"), Some(512 * 1024 * 1024));
        assert_eq!(parse_mem_limit("1024k"), Some(1024 * 1024));
        assert_eq!(parse_mem_limit("1048576"), Some(1048576));
        assert_eq!(parse_mem_limit("bad"), None);
    }
}
