use anyhow::{bail, Result};

use crate::config;
use crate::docker::{self, DockerCompose};
use crate::output;
use crate::state::InstanceState;

pub fn run(name: &str, follow: bool, tail: &str) -> Result<()> {
    let state = InstanceState::require(name)?;

    if state.isolation == "process" {
        output::info(&format!("Instance '{}' uses process mode — no container logs available.", name));
        output::info("Check your system process logs instead.");
        return Ok(());
    }

    docker::require_docker()?;

    let compose_path = config::instance_dir(name).join("docker-compose.yml");
    if !compose_path.exists() {
        bail!("No docker-compose.yml found for instance '{}'", name);
    }

    let dc = DockerCompose::new(name, &compose_path.to_string_lossy());
    dc.logs(follow, tail)?;
    Ok(())
}
