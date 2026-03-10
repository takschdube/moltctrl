use std::process::Command;

use anyhow::{bail, Context, Result};

use crate::config;
use crate::output;

/// Check if Docker is installed
pub fn is_docker_installed() -> bool {
    Command::new("docker")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Check if Docker daemon is running
pub fn is_docker_running() -> bool {
    Command::new("docker")
        .arg("info")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Check if Docker Compose v2 is available
pub fn is_compose_available() -> bool {
    Command::new("docker")
        .args(["compose", "version"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Get Docker version string
pub fn docker_version() -> Option<String> {
    Command::new("docker")
        .arg("--version")
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                String::from_utf8(o.stdout)
                    .ok()
                    .map(|s| s.trim().to_string())
            } else {
                None
            }
        })
}

/// Get Docker Compose version string
pub fn compose_version() -> Option<String> {
    Command::new("docker")
        .args(["compose", "version", "--short"])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                String::from_utf8(o.stdout)
                    .ok()
                    .map(|s| s.trim().to_string())
            } else {
                None
            }
        })
}

/// Require Docker to be installed and running
pub fn require_docker() -> Result<()> {
    if !is_docker_installed() {
        bail!("Docker is not installed. Run 'moltctrl doctor' or install Docker first.");
    }
    if !is_docker_running() {
        bail!("Docker daemon is not running or you don't have permission. Try 'sudo systemctl start docker' or add yourself to the docker group.");
    }
    Ok(())
}

/// Docker Compose wrapper for a named instance
pub struct DockerCompose {
    project: String,
    compose_file: String,
}

impl DockerCompose {
    pub fn new(name: &str, compose_file: &str) -> Self {
        Self {
            project: format!("{}-{}", config::COMPOSE_PREFIX, name),
            compose_file: compose_file.to_string(),
        }
    }

    /// Run a docker compose command
    fn run(&self, args: &[&str]) -> Result<std::process::Output> {
        let output = Command::new("docker")
            .arg("compose")
            .arg("-p")
            .arg(&self.project)
            .arg("-f")
            .arg(&self.compose_file)
            .args(args)
            .output()
            .context("Failed to run docker compose")?;
        Ok(output)
    }

    /// Run docker compose up -d
    pub fn up(&self) -> Result<()> {
        let output = self.run(&["up", "-d"])?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            output::debug(&format!("docker compose up stderr: {}", stderr));
            bail!("docker compose up failed");
        }
        Ok(())
    }

    /// Run docker compose stop
    pub fn stop(&self) -> Result<()> {
        let output = self.run(&["stop"])?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            output::debug(&format!("docker compose stop stderr: {}", stderr));
            bail!("docker compose stop failed");
        }
        Ok(())
    }

    /// Run docker compose restart
    pub fn restart(&self) -> Result<()> {
        let output = self.run(&["restart"])?;
        if !output.status.success() {
            bail!("docker compose restart failed");
        }
        Ok(())
    }

    /// Run docker compose down -v --remove-orphans
    pub fn down(&self) -> Result<()> {
        let _ = self.run(&["down", "-v", "--remove-orphans"]);
        Ok(())
    }

    /// Run docker compose logs
    pub fn logs(&self, follow: bool, tail: &str) -> Result<()> {
        let mut cmd = Command::new("docker");
        cmd.arg("compose")
            .arg("-p")
            .arg(&self.project)
            .arg("-f")
            .arg(&self.compose_file)
            .arg("logs")
            .arg("--tail")
            .arg(tail);
        if follow {
            cmd.arg("-f");
        }
        let status = cmd.status().context("Failed to run docker compose logs")?;
        if !status.success() {
            bail!("docker compose logs failed");
        }
        Ok(())
    }

    /// Get container status via docker compose ps --format json
    pub fn status(&self) -> Result<Option<String>> {
        let output = self.run(&["ps", "--format", "json"])?;
        if !output.status.success() {
            return Ok(None);
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stdout = stdout.trim();
        if stdout.is_empty() {
            return Ok(Some("stopped".to_string()));
        }
        // Parse first JSON line
        if let Some(line) = stdout.lines().next() {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(line) {
                let state = val["State"].as_str().unwrap_or("unknown");
                return Ok(Some(state.to_string()));
            }
        }
        Ok(Some("unknown".to_string()))
    }

    /// Get container health via docker compose ps --format json
    pub fn health(&self) -> Result<Option<String>> {
        let output = self.run(&["ps", "--format", "json"])?;
        if !output.status.success() {
            return Ok(None);
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stdout = stdout.trim();
        if stdout.is_empty() {
            return Ok(Some("unknown".to_string()));
        }
        if let Some(line) = stdout.lines().next() {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(line) {
                let health = val["Health"].as_str().unwrap_or("unknown");
                return Ok(Some(health.to_string()));
            }
        }
        Ok(Some("unknown".to_string()))
    }

    /// Pull the Docker image
    pub fn pull_image(image: &str) -> Result<bool> {
        let output = Command::new("docker")
            .args(["pull", image])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .output()
            .context("Failed to pull Docker image")?;
        Ok(output.status.success())
    }
}
