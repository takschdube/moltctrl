use std::fs::{self, File, OpenOptions};
use std::io::BufRead;
use std::path::Path;
use std::process::Command;

use anyhow::{bail, Context, Result};

use crate::config;
#[cfg(unix)]
use crate::sandbox::{self, parse_mem_limit, SandboxConfig};
use crate::state::InstanceState;

/// Parse a `.env` file and return key-value pairs.
/// Skips empty lines and lines starting with `#`.
fn parse_env_file(path: &Path) -> Result<Vec<(String, String)>> {
    let file =
        File::open(path).with_context(|| format!("Failed to open .env file at {:?}", path))?;
    let reader = std::io::BufReader::new(file);
    let mut vars = Vec::new();

    for line in reader.lines() {
        let line = line.context("Failed to read line from .env file")?;
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = trimmed.split_once('=') {
            let key = key.trim().to_string();
            let value = value.trim().to_string();
            if !key.is_empty() {
                vars.push((key, value));
            }
        }
    }

    Ok(vars)
}

/// Resolve the OpenClaw command. Returns `(program, extra_args)`.
///
/// Resolution order:
/// 1. `openclaw` binary on PATH (standalone install)
/// 2. `npx @openclaw/openclaw` (if npx is available via Node.js)
/// 3. Error with installation instructions
fn resolve_openclaw_command() -> Result<(String, Vec<String>)> {
    if which_exists("openclaw") {
        return Ok(("openclaw".to_string(), Vec::new()));
    }
    if which_exists("npx") {
        return Ok(("npx".to_string(), vec!["@openclaw/openclaw".to_string()]));
    }
    bail!(
        "OpenClaw is not installed. Process mode requires the OpenClaw binary.\n\
         Install it with: npm install -g @openclaw/openclaw\n\
         Or use Docker mode: moltctrl create <name> --docker"
    )
}

/// Check if a command exists on PATH.
///
/// Uses `where` on Windows and `which` on Unix-like systems.
fn which_exists(cmd: &str) -> bool {
    #[cfg(windows)]
    let lookup = Command::new("where")
        .arg(cmd)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    #[cfg(not(windows))]
    let lookup = Command::new("which")
        .arg(cmd)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    lookup.map(|s| s.success()).unwrap_or(false)
}

/// Spawn an OpenClaw process in the background with sandbox resource limits.
///
/// Loads environment variables from the instance's `.env` file, sets `PORT`,
/// redirects stdout/stderr to the log file, and applies sandbox resource limits
/// via `pre_exec` on Unix.
///
/// Returns the child process PID.
pub fn spawn_process(state: &InstanceState, log_path: &Path) -> Result<u32> {
    let inst_dir = config::instance_dir(&state.name);
    let env_file = inst_dir.join(".env");

    // Parse environment variables from the .env file
    let env_vars = if env_file.exists() {
        parse_env_file(&env_file)?
    } else {
        Vec::new()
    };

    let (program, extra_args) = resolve_openclaw_command()?;

    // Open/create the log file for stdout/stderr redirection
    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
        .with_context(|| format!("Failed to open log file at {:?}", log_path))?;
    let log_stderr = log_file
        .try_clone()
        .context("Failed to clone log file handle for stderr")?;

    // Ensure instance directory exists
    fs::create_dir_all(&inst_dir)
        .with_context(|| format!("Failed to create instance directory {:?}", inst_dir))?;

    let mut cmd = Command::new(&program);
    cmd.args(&extra_args);
    cmd.current_dir(&inst_dir);
    cmd.stdout(log_file);
    cmd.stderr(log_stderr);

    // Set PORT env var
    cmd.env("PORT", state.port.to_string());

    // Set environment variables from .env file
    for (key, value) in &env_vars {
        cmd.env(key, value);
    }

    // Apply sandbox resource limits via pre_exec on Unix
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;

        let sandbox_config = SandboxConfig {
            working_dir: inst_dir.clone(),
            mem_limit_bytes: parse_mem_limit(&state.mem),
            cpu_limit: state.cpus.parse::<u32>().ok(),
            pid_limit: state.pids.parse::<u32>().ok(),
        };

        unsafe {
            cmd.pre_exec(move || {
                let sb = sandbox::create_sandbox();
                sb.apply(&sandbox_config).map_err(std::io::Error::other)?;
                Ok(())
            });
        }
    }

    let child = cmd.spawn().with_context(|| {
        format!(
            "Failed to spawn '{program}'. If OpenClaw is not installed, run:\n  \
             npm install -g @openclaw/openclaw\n\
             Or use Docker mode: moltctrl create <name> --docker"
        )
    })?;

    Ok(child.id())
}

/// Kill a process by PID.
///
/// On Unix: sends SIGTERM, waits up to 5 seconds, then sends SIGKILL if still alive.
/// On Windows: uses `taskkill /PID <pid> /F`.
pub fn kill_process(pid: u32) -> Result<()> {
    #[cfg(unix)]
    {
        use nix::sys::signal::{self, Signal};
        use nix::unistd::Pid;
        use std::thread;
        use std::time::Duration;

        let nix_pid = Pid::from_raw(pid as i32);

        // Send SIGTERM for graceful shutdown
        match signal::kill(nix_pid, Signal::SIGTERM) {
            Ok(()) => {}
            Err(nix::errno::Errno::ESRCH) => {
                // Process already gone
                return Ok(());
            }
            Err(e) => {
                bail!("Failed to send SIGTERM to PID {}: {}", pid, e);
            }
        }

        // Wait up to 5 seconds for the process to exit
        for _ in 0..50 {
            thread::sleep(Duration::from_millis(100));
            if !is_process_running(pid) {
                return Ok(());
            }
        }

        // Process still alive after 5s — send SIGKILL
        match signal::kill(nix_pid, Signal::SIGKILL) {
            Ok(()) => {}
            Err(nix::errno::Errno::ESRCH) => {
                return Ok(());
            }
            Err(e) => {
                bail!("Failed to send SIGKILL to PID {}: {}", pid, e);
            }
        }

        Ok(())
    }

    #[cfg(windows)]
    {
        let status = Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/F"])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .context("Failed to execute taskkill")?;

        if !status.success() {
            bail!(
                "taskkill failed for PID {} (exit code {:?})",
                pid,
                status.code()
            );
        }

        Ok(())
    }

    #[cfg(not(any(unix, windows)))]
    {
        bail!("Process killing is not supported on this platform");
    }
}

/// Check if a process with the given PID is still running.
///
/// On Unix: uses `kill(pid, 0)` which succeeds if the process exists.
/// On Windows: checks via `tasklist` filtered by PID.
pub fn is_process_running(pid: u32) -> bool {
    #[cfg(unix)]
    {
        use nix::sys::signal;
        use nix::unistd::Pid;

        let nix_pid = Pid::from_raw(pid as i32);
        // Sending signal 0 checks if the process exists without delivering a signal
        signal::kill(nix_pid, None).is_ok()
    }

    #[cfg(windows)]
    {
        Command::new("tasklist")
            .args(["/FI", &format!("PID eq {}", pid), "/NH"])
            .output()
            .map(|output| {
                let stdout = String::from_utf8_lossy(&output.stdout);
                stdout.contains(&pid.to_string())
            })
            .unwrap_or(false)
    }

    #[cfg(not(any(unix, windows)))]
    {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_env_file_basic() {
        let dir = tempfile::tempdir().unwrap();
        let env_path = dir.path().join(".env");
        fs::write(
            &env_path,
            "# comment\n\nANTHROPIC_API_KEY=sk-ant-test\nOPENCLAW_PROVIDER=anthropic\nOPENCLAW_MODEL=claude-sonnet-4-20250514\n",
        )
        .unwrap();

        let vars = parse_env_file(&env_path).unwrap();
        assert_eq!(vars.len(), 3);
        assert_eq!(vars[0], ("ANTHROPIC_API_KEY".into(), "sk-ant-test".into()));
        assert_eq!(vars[1], ("OPENCLAW_PROVIDER".into(), "anthropic".into()));
        assert_eq!(
            vars[2],
            ("OPENCLAW_MODEL".into(), "claude-sonnet-4-20250514".into())
        );
    }

    #[test]
    fn test_parse_env_file_skips_comments_and_empty() {
        let dir = tempfile::tempdir().unwrap();
        let env_path = dir.path().join(".env");
        fs::write(
            &env_path,
            "# full line comment\n\n  \n# another comment\nKEY=value\n",
        )
        .unwrap();

        let vars = parse_env_file(&env_path).unwrap();
        assert_eq!(vars.len(), 1);
        assert_eq!(vars[0], ("KEY".into(), "value".into()));
    }

    #[test]
    fn test_parse_env_file_missing() {
        let result = parse_env_file(Path::new("/nonexistent/.env"));
        assert!(result.is_err());
    }

    #[test]
    fn test_is_process_running_self() {
        let pid = std::process::id();
        assert!(is_process_running(pid));
    }

    #[test]
    fn test_is_process_running_nonexistent() {
        // PID 4194304 is above typical PID max and unlikely to exist
        assert!(!is_process_running(4_194_304));
    }
}
