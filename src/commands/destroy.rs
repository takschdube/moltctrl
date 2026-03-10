use std::fs;
use std::io::IsTerminal;

use anyhow::Result;
use dialoguer::Confirm;

use crate::config;
use crate::docker::{self, DockerCompose};
use crate::output;
use crate::state::InstanceState;

pub fn run(name: &str, force: bool, global_force: bool) -> Result<()> {
    let _state = InstanceState::require(name)?;

    if !force && !global_force {
        if !std::io::stdin().is_terminal() {
            anyhow::bail!("Cannot prompt for confirmation in non-interactive mode. Use --force.");
        }
        let confirmed = Confirm::new()
            .with_prompt(format!("Destroy instance '{}' and all its data?", name))
            .default(false)
            .interact()?;
        if !confirmed {
            output::info("Aborted.");
            return Ok(());
        }
    }

    output::info(&format!("Destroying instance '{}'...", name));

    // Stop and remove containers + volumes if docker is available
    if docker::is_docker_installed() && docker::is_docker_running() {
        let compose_path = config::instance_dir(name).join("docker-compose.yml");
        if compose_path.exists() {
            let spinner = output::spinner("Removing containers and volumes...");
            let dc = DockerCompose::new(name, &compose_path.to_string_lossy());
            let _ = dc.down();
            spinner.finish_and_clear();
        }
    }

    // Remove instance directory
    let inst_dir = config::instance_dir(name);
    if inst_dir.exists() {
        fs::remove_dir_all(&inst_dir)?;
    }

    output::success(&format!("Instance '{}' destroyed", name));
    Ok(())
}
