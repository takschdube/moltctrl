use anyhow::Result;

use crate::config;
use crate::docker::{self, DockerCompose};
use crate::output;
use crate::state::{self, InstanceState};

pub fn run() -> Result<()> {
    config::ensure_dirs()?;

    let names = state::list_names()?;

    if names.is_empty() {
        output::info("No instances found. Create one with: moltctrl create <name>");
        return Ok(());
    }

    let header = format!(
        "{:<20} {:<12} {:<12} {:<8} {:<10} CREATED",
        "NAME", "PROVIDER", "STATUS", "PORT", "HEALTH"
    );
    println!("{}", header);
    let sep = format!(
        "{:<20} {:<12} {:<12} {:<8} {:<10} -------",
        "----", "--------", "------", "----", "------"
    );
    println!("{}", sep);

    let docker_available = docker::is_docker_installed() && docker::is_docker_running();

    for name in &names {
        let instance = match InstanceState::load(name) {
            Ok(i) => i,
            Err(_) => continue,
        };

        let mut status = instance.status.clone();
        let mut health = "-".to_string();

        if docker_available {
            let compose_path = config::instance_dir(name).join("docker-compose.yml");
            if compose_path.exists() {
                let dc = DockerCompose::new(name, &compose_path.to_string_lossy());
                if let Ok(Some(live_status)) = dc.status() {
                    if live_status != "stopped" {
                        status = live_status;
                    }
                }
                if let Ok(Some(h)) = dc.health() {
                    health = h;
                }
            }
        }

        // Truncate created date
        let created = instance
            .created
            .split('T')
            .next()
            .unwrap_or(&instance.created);

        println!(
            "{:<20} {:<12} {:<12} {:<8} {:<10} {}",
            instance.name, instance.provider, status, instance.port, health, created
        );
    }

    Ok(())
}
