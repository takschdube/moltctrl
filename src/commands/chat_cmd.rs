use anyhow::{bail, Result};

use crate::chat;
use crate::config;
use crate::docker::DockerCompose;
use crate::state::InstanceState;

pub async fn run(name: &str) -> Result<()> {
    let mut state = InstanceState::require(name)?;

    // Prefer OpenClaw's token if available (onboard may have generated its own)
    if let Some(oc_token) = crate::process_manager::read_openclaw_token(name) {
        state.token = oc_token;
    }

    // Check if the instance is running
    let compose_path = config::instance_dir(name).join("docker-compose.yml");
    if compose_path.exists() {
        let dc = DockerCompose::new(name, &compose_path.to_string_lossy());
        if let Ok(Some(status)) = dc.status() {
            if status != "running" {
                bail!(
                    "Instance '{}' is not running (status: {}). Start it first.",
                    name,
                    status
                );
            }
        }
    }

    chat::start_chat(state.port, &state.token).await
}
