use anyhow::{Context, Result};
use nix::sys::resource::{setrlimit, Resource};
use nix::unistd;

use crate::sandbox::{ProcessSandbox, SandboxConfig};

#[derive(Default)]
pub struct UnixSandbox;

impl UnixSandbox {
    pub fn new() -> Self {
        Self
    }
}

impl ProcessSandbox for UnixSandbox {
    fn apply(&self, config: &SandboxConfig) -> Result<()> {
        // Change to working directory
        unistd::chdir(&config.working_dir)
            .with_context(|| format!("Failed to chdir to {:?}", config.working_dir))?;

        // Set memory limit (RLIMIT_AS - address space)
        if let Some(mem_bytes) = config.mem_limit_bytes {
            setrlimit(Resource::RLIMIT_AS, mem_bytes, mem_bytes)
                .context("Failed to set RLIMIT_AS")?;
        }

        // Set process limit (RLIMIT_NPROC) — Linux only
        #[cfg(target_os = "linux")]
        if let Some(pid_limit) = config.pid_limit {
            let limit = pid_limit as u64;
            setrlimit(Resource::RLIMIT_NPROC, limit, limit)
                .context("Failed to set RLIMIT_NPROC")?;
        }
        #[cfg(not(target_os = "linux"))]
        let _ = config.pid_limit; // suppress unused warning on macOS

        // Set CPU time limit (RLIMIT_CPU) - generous default
        if let Some(cpu_limit) = config.cpu_limit {
            // Use cpu_limit as hours of CPU time
            let seconds = cpu_limit as u64 * 3600;
            setrlimit(Resource::RLIMIT_CPU, seconds, seconds)
                .context("Failed to set RLIMIT_CPU")?;
        }

        // Limit file size (RLIMIT_FSIZE) - 1GB
        let fsize_limit = 1024 * 1024 * 1024;
        let _ = setrlimit(Resource::RLIMIT_FSIZE, fsize_limit, fsize_limit);

        // Limit core dump size to 0
        let _ = setrlimit(Resource::RLIMIT_CORE, 0, 0);

        Ok(())
    }

    fn cleanup(&self) -> Result<()> {
        // No persistent resources to clean up for ulimit-based sandboxing
        Ok(())
    }
}
