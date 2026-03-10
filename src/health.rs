use std::time::Duration;

use anyhow::Result;

use crate::config;
use crate::docker::DockerCompose;
use crate::output;

/// Wait for an instance to become healthy by polling docker compose health
pub async fn wait_healthy(name: &str, timeout_secs: u64) -> Result<bool> {
    let compose_file = config::instance_dir(name)
        .join("docker-compose.yml")
        .to_string_lossy()
        .to_string();
    let dc = DockerCompose::new(name, &compose_file);

    let interval = Duration::from_secs(3);
    let mut elapsed = Duration::ZERO;
    let timeout = Duration::from_secs(timeout_secs);

    let spinner = output::spinner(&format!("Waiting for {} to become healthy...", name));

    while elapsed < timeout {
        // Check health
        if let Ok(Some(health)) = dc.health() {
            if health == "healthy" {
                spinner.finish_and_clear();
                return Ok(true);
            }
        }
        // Check if container exited
        if let Ok(Some(status)) = dc.status() {
            if status == "exited" || status == "dead" {
                spinner.finish_and_clear();
                return Ok(false);
            }
        }
        tokio::time::sleep(interval).await;
        elapsed += interval;
    }

    spinner.finish_and_clear();
    Ok(false)
}
