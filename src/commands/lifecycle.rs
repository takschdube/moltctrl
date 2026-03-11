use anyhow::{bail, Result};

use crate::config;
use crate::docker::{self, DockerCompose};
use crate::output;
use crate::state::InstanceState;

pub fn start(name: &str) -> Result<()> {
    let mut state = InstanceState::require(name)?;

    if state.isolation == "process" {
        // Process mode — just update status
        // TODO: actually spawn the process with sandbox limits
        state.status = "running".to_string();
        state.save()?;
        output::success(&format!("Instance '{}' started (process mode)", name));
        return Ok(());
    }

    docker::require_docker()?;

    let compose_path = config::instance_dir(name).join("docker-compose.yml");
    if !compose_path.exists() {
        bail!("No docker-compose.yml found for instance '{}'", name);
    }

    output::info(&format!("Starting instance '{}'...", name));
    let spinner = output::spinner("Starting containers...");
    let dc = DockerCompose::new(name, &compose_path.to_string_lossy());
    match dc.up() {
        Ok(()) => {
            spinner.finish_and_clear();
            state.status = "running".to_string();
            state.save()?;
            output::success(&format!("Instance '{}' started", name));
        }
        Err(_) => {
            spinner.finish_and_clear();
            bail!("Failed to start instance '{}'", name);
        }
    }
    Ok(())
}

pub fn stop(name: &str) -> Result<()> {
    let mut state = InstanceState::require(name)?;

    if state.isolation == "process" {
        state.status = "stopped".to_string();
        state.save()?;
        output::success(&format!("Instance '{}' stopped (process mode)", name));
        return Ok(());
    }

    docker::require_docker()?;

    let compose_path = config::instance_dir(name).join("docker-compose.yml");
    if !compose_path.exists() {
        bail!("No docker-compose.yml found for instance '{}'", name);
    }

    output::info(&format!("Stopping instance '{}'...", name));
    let spinner = output::spinner("Stopping containers...");
    let dc = DockerCompose::new(name, &compose_path.to_string_lossy());
    match dc.stop() {
        Ok(()) => {
            spinner.finish_and_clear();
            state.status = "stopped".to_string();
            state.save()?;
            output::success(&format!("Instance '{}' stopped", name));
        }
        Err(_) => {
            spinner.finish_and_clear();
            bail!("Failed to stop instance '{}'", name);
        }
    }
    Ok(())
}

pub fn restart(name: &str) -> Result<()> {
    let mut state = InstanceState::require(name)?;

    if state.isolation == "process" {
        state.status = "running".to_string();
        state.save()?;
        output::success(&format!("Instance '{}' restarted (process mode)", name));
        return Ok(());
    }

    docker::require_docker()?;

    let compose_path = config::instance_dir(name).join("docker-compose.yml");
    if !compose_path.exists() {
        bail!("No docker-compose.yml found for instance '{}'", name);
    }

    output::info(&format!("Restarting instance '{}'...", name));
    let spinner = output::spinner("Restarting containers...");
    let dc = DockerCompose::new(name, &compose_path.to_string_lossy());
    match dc.restart() {
        Ok(()) => {
            spinner.finish_and_clear();
            state.status = "running".to_string();
            state.save()?;
            output::success(&format!("Instance '{}' restarted", name));
        }
        Err(_) => {
            spinner.finish_and_clear();
            bail!("Failed to restart instance '{}'", name);
        }
    }
    Ok(())
}
