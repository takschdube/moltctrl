use anyhow::Result;

use crate::config;
use crate::docker::{self, DockerCompose};
use crate::state::InstanceState;

pub fn run(name: &str) -> Result<()> {
    let instance = InstanceState::require(name)?;

    println!("Instance: {}", instance.name);
    println!("Provider: {}", instance.provider);
    println!("Model:    {}", instance.model);
    println!("Image:    {}", instance.image);
    println!("Port:     {}", instance.port);
    println!("Created:  {}", instance.created);
    println!("Mem:      {}", instance.mem);
    println!("CPUs:     {}", instance.cpus);
    if !instance.isolation.is_empty() {
        println!("Isolation: {}", instance.isolation);
    }
    println!();

    if docker::is_docker_installed() && docker::is_docker_running() {
        let compose_path = config::instance_dir(name).join("docker-compose.yml");
        if compose_path.exists() {
            let dc = DockerCompose::new(name, &compose_path.to_string_lossy());
            let status = dc
                .status()
                .ok()
                .flatten()
                .unwrap_or_else(|| "unknown".to_string());
            let health = dc
                .health()
                .ok()
                .flatten()
                .unwrap_or_else(|| "unknown".to_string());
            println!("Docker Status: {}", status);
            println!("Health:        {}", health);
        } else {
            println!("Docker Status: (no compose file)");
        }
    } else {
        println!("Docker Status: (docker not available)");
    }

    Ok(())
}
