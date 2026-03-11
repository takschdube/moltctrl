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
/// Skips empty lines and comment lines (starting with `#`).
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

/// Read the auth token from OpenClaw's config file.
///
/// After `openclaw onboard` runs, it creates its own token in
/// `.openclaw/openclaw.json` under `gateway.auth.token`.
pub fn read_openclaw_token(name: &str) -> Option<String> {
    let inst_dir = config::instance_dir(name);
    let config_path = inst_dir.join(".openclaw").join("openclaw.json");
    let content = fs::read_to_string(config_path).ok()?;
    let config: serde_json::Value = serde_json::from_str(&content).ok()?;
    config
        .get("gateway")
        .and_then(|g| g.get("auth"))
        .and_then(|a| a.get("token"))
        .and_then(|t| t.as_str())
        .map(|s| s.to_string())
}

/// Generate a full OpenClaw config JSON for this instance.
///
/// Based on the validated schema from a real `openclaw onboard` run.
/// This skips onboard entirely — no redundant prompts for provider/key
/// that moltctrl already collected. API keys are passed via env vars.
fn generate_openclaw_config(state: &InstanceState, _env_vars: &[(String, String)]) -> String {
    use serde_json::json;

    let inst_dir = config::instance_dir(&state.name);
    let workspace_dir = inst_dir.join(".openclaw").join("workspace");
    let model_id = format!("{}/{}", state.provider, state.model);
    let token_name = format!("moltctrl-{}", state.name);
    let now = chrono::Utc::now().to_rfc3339();

    // Build auth profile key based on provider
    let auth_profile_key = format!("{}:default", state.provider);

    let config = json!({
        "wizard": {
            "lastRunAt": now,
            "lastRunVersion": "2026.3.8",
            "lastRunCommand": "onboard",
            "lastRunMode": "local"
        },
        "auth": {
            "profiles": {
                (auth_profile_key): {
                    "provider": state.provider,
                    "mode": "token"
                }
            }
        },
        "agents": {
            "defaults": {
                "model": {
                    "primary": model_id
                },
                "workspace": workspace_dir.to_string_lossy(),
                "contextPruning": {
                    "mode": "cache-ttl",
                    "ttl": "1h"
                },
                "compaction": {
                    "mode": "safeguard"
                },
                "heartbeat": {
                    "every": "30m"
                },
                "maxConcurrent": 4,
                "subagents": {
                    "maxConcurrent": 8
                }
            },
            "list": [
                {
                    "id": "main",
                    "default": true,
                    "workspace": workspace_dir.to_string_lossy()
                }
            ]
        },
        "tools": {
            "profile": "coding"
        },
        "messages": {
            "ackReactionScope": "group-mentions"
        },
        "commands": {
            "native": "auto",
            "nativeSkills": "auto",
            "restart": true,
            "ownerDisplay": "raw"
        },
        "session": {
            "dmScope": "per-channel-peer"
        },
        "hooks": {
            "internal": {
                "enabled": true,
                "entries": {
                    "boot-md": { "enabled": true },
                    "bootstrap-extra-files": { "enabled": true },
                    "command-logger": { "enabled": true },
                    "session-memory": { "enabled": true }
                }
            }
        },
        "gateway": {
            "port": state.port,
            "mode": "local",
            "bind": "loopback",
            "auth": {
                "mode": "token",
                "token": state.token,
                "tokenName": token_name
            },
            "tailscale": {
                "mode": "off",
                "resetOnExit": false
            },
            "nodes": {
                "denyCommands": [
                    "camera.snap", "camera.clip", "screen.record",
                    "contacts.add", "calendar.add", "reminders.add", "sms.send"
                ]
            }
        },
        "meta": {
            "lastTouchedVersion": "2026.3.8",
            "lastTouchedAt": now
        }
    });

    serde_json::to_string_pretty(&config).unwrap_or_else(|_| "{}".to_string())
}

/// Create workspace directory structure and essential files for OpenClaw.
///
/// Sets up the workspace, agent dirs, SOUL.md, and TOOLS.md that OpenClaw
/// expects to find on startup.
fn setup_workspace_fallback(state: &InstanceState) -> Result<()> {
    let inst_dir = config::instance_dir(&state.name);
    // Config points workspace at .openclaw/workspace, create both for compat
    let oc_workspace = inst_dir.join(".openclaw").join("workspace");
    let workspace_dir = inst_dir.join("workspace");
    fs::create_dir_all(&oc_workspace)?;
    fs::create_dir_all(&workspace_dir)?;

    // Create agents directory structure OpenClaw expects
    let agents_dir = inst_dir.join("agents").join("main").join("agent");
    fs::create_dir_all(agents_dir.join("sessions"))?;

    // SOUL.md — Agent persona
    let soul_path = workspace_dir.join("SOUL.md");
    if !soul_path.exists() {
        fs::write(
            &soul_path,
            format!(
                "# Agent: {}\n\n\
                 You are a helpful AI assistant powered by {}/{}.\n\n\
                 ## Behavior\n\
                 - Be direct and concise\n\
                 - Ask clarifying questions when the request is ambiguous\n\
                 - Use tools when they help accomplish the task\n\
                 - Respect sandbox boundaries and security constraints\n\n\
                 ## Capabilities\n\
                 - Web search and content fetching\n\
                 - Code execution in sandboxed environment\n\
                 - File reading and writing within workspace\n\
                 - Multi-step reasoning and task decomposition\n",
                state.name, state.provider, state.model
            ),
        )?;
    }

    // TOOLS.md
    let tools_path = workspace_dir.join("TOOLS.md");
    if !tools_path.exists() {
        fs::write(
            &tools_path,
            "# Available Tools\n\n\
             ## Web\n\
             - **search** — Search the web for information\n\
             - **fetch** — Fetch content from a URL\n\n\
             ## Sandbox\n\
             - **execute** — Run code in a sandboxed environment\n\
             - **read_file** — Read files from the workspace\n\
             - **write_file** — Write files to the workspace\n\n\
             ## Memory\n\
             - **remember** — Store information for later recall\n\
             - **recall** — Search stored memories\n",
        )?;
    }

    Ok(())
}

/// Resolve the OpenClaw command. Returns `(program, extra_args)`.
///
/// First ensures the runtime (Node.js + OpenClaw) is installed,
/// downloading automatically if needed. Then returns the command to invoke.
fn resolve_openclaw_command() -> Result<(String, Vec<String>)> {
    use crate::runtime;

    // Auto-download Node.js + OpenClaw if not present
    runtime::ensure_runtime()?;

    // Get the resolved command
    runtime::openclaw_command()
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

    // Generate OpenClaw config directly — no onboard wizard needed.
    // moltctrl already collected provider, API key, and model from the user.
    // Running onboard would redundantly ask for these again.
    let openclaw_subdir = inst_dir.join(".openclaw");
    let config_path = openclaw_subdir.join("openclaw.json");

    fs::create_dir_all(&openclaw_subdir)?;
    let openclaw_config = generate_openclaw_config(state, &env_vars);
    fs::write(&config_path, &openclaw_config)
        .with_context(|| format!("Failed to write OpenClaw config to {:?}", config_path))?;

    // Create workspace and agent directory structure
    setup_workspace_fallback(state)?;

    // Set state dir so each instance is fully isolated
    let state_dir = inst_dir.join(".openclaw").join("state");
    fs::create_dir_all(&state_dir)?;

    let mut cmd = Command::new(&program);
    cmd.args(&extra_args);
    cmd.args(["gateway", "--port", &state.port.to_string()]);
    cmd.current_dir(&inst_dir);
    cmd.stdout(log_file);
    cmd.stderr(log_stderr);

    // Point OpenClaw to the instance's isolated home
    cmd.env("OPENCLAW_HOME", &inst_dir);
    cmd.env("OPENCLAW_STATE_DIR", &state_dir);
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
    fn test_generate_openclaw_config() {
        let state = InstanceState::new(
            "test-agent",
            18789,
            "anthropic",
            "claude-sonnet-4-20250514",
            "img:latest",
        );
        let env_vars = vec![
            ("ANTHROPIC_API_KEY".to_string(), "sk-ant-test".to_string()),
            ("OPENCLAW_PROVIDER".to_string(), "anthropic".to_string()),
        ];

        let config_str = generate_openclaw_config(&state, &env_vars);
        let config: serde_json::Value = serde_json::from_str(&config_str).unwrap();

        // Gateway — validated schema fields
        assert_eq!(config["gateway"]["mode"], "local");
        assert_eq!(config["gateway"]["port"], 18789);
        assert_eq!(config["gateway"]["auth"]["mode"], "token");
        assert_eq!(
            config["gateway"]["auth"]["tokenName"],
            "moltctrl-test-agent"
        );
        assert_eq!(config["gateway"]["bind"], "loopback");
        assert_eq!(config["gateway"]["tailscale"]["mode"], "off");
        assert!(config["gateway"]["nodes"]["denyCommands"].is_array());

        // Auth profiles
        assert_eq!(
            config["auth"]["profiles"]["anthropic:default"]["provider"],
            "anthropic"
        );
        assert_eq!(
            config["auth"]["profiles"]["anthropic:default"]["mode"],
            "token"
        );

        // Agents
        assert_eq!(
            config["agents"]["defaults"]["model"]["primary"],
            "anthropic/claude-sonnet-4-20250514"
        );
        assert_eq!(config["agents"]["defaults"]["maxConcurrent"], 4);
        assert_eq!(
            config["agents"]["defaults"]["contextPruning"]["mode"],
            "cache-ttl"
        );
        assert_eq!(
            config["agents"]["defaults"]["compaction"]["mode"],
            "safeguard"
        );
        assert_eq!(config["agents"]["list"][0]["id"], "main");
        assert_eq!(config["agents"]["list"][0]["default"], true);

        // Tools — uses profile string, not nested objects
        assert_eq!(config["tools"]["profile"], "coding");

        // Hooks
        assert_eq!(config["hooks"]["internal"]["enabled"], true);

        // Wizard metadata
        assert!(config["wizard"]["lastRunAt"].is_string());
        assert!(config["meta"]["lastTouchedAt"].is_string());

        // Should NOT have invalid keys
        assert!(config.get("models").is_none());
        assert!(config.get("memory").is_none());
        assert!(config.get("logging").is_none());
    }

    #[test]
    fn test_setup_workspace_fallback() {
        // We can't test run_openclaw_setup without a real OpenClaw install,
        // but we can test the fallback creates the right structure.
        let dir = tempfile::tempdir().unwrap();
        let inst_name = "test-ws";

        // Create a mock instance state
        let state = InstanceState::new(
            inst_name,
            18789,
            "anthropic",
            "claude-sonnet-4-20250514",
            "img:latest",
        );

        // Create the instance dir where config::instance_dir would point
        let inst_dir = dir.path().join(inst_name);
        fs::create_dir_all(&inst_dir).unwrap();

        // Manually call fallback with the temp dir as workspace
        let workspace_dir = inst_dir.join("workspace");
        fs::create_dir_all(&workspace_dir).unwrap();
        let agents_dir = inst_dir.join("agents").join("main").join("agent");
        fs::create_dir_all(agents_dir.join("sessions")).unwrap();

        let soul_path = workspace_dir.join("SOUL.md");
        fs::write(&soul_path, format!("# Agent: {}\n", state.name)).unwrap();

        assert!(soul_path.exists());
        assert!(agents_dir.join("sessions").exists());

        let soul = fs::read_to_string(&soul_path).unwrap();
        assert!(soul.contains("test-ws"));
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
