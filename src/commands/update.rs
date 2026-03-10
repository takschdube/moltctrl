use std::fs;

use anyhow::{bail, Result};

use crate::config;
use crate::output;
use crate::state::InstanceState;
use crate::template::{self, TemplateVars};

pub fn run(
    name: &str,
    model: Option<&str>,
    mem: Option<&str>,
    cpus: Option<&str>,
    pids: Option<&str>,
) -> Result<()> {
    let mut state = InstanceState::require(name)?;

    let mut changed = false;

    if let Some(m) = model {
        state.model = m.to_string();

        // Update .env file
        let env_file = config::instance_dir(name).join(".env");
        if env_file.exists() {
            let content = fs::read_to_string(&env_file)?;
            let updated = content
                .lines()
                .map(|line| {
                    if line.starts_with("OPENCLAW_MODEL=") {
                        format!("OPENCLAW_MODEL={}", m)
                    } else {
                        line.to_string()
                    }
                })
                .collect::<Vec<_>>()
                .join("\n");
            fs::write(&env_file, updated)?;

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(&env_file, fs::Permissions::from_mode(0o600))?;
            }
        }

        output::info(&format!("Model updated to: {}", m));
        changed = true;
    }

    if let Some(m) = mem {
        state.mem = m.to_string();
        output::info(&format!("Memory limit updated to: {}", m));
        changed = true;
    }

    if let Some(c) = cpus {
        state.cpus = c.to_string();
        output::info(&format!("CPU limit updated to: {}", c));
        changed = true;
    }

    if let Some(p) = pids {
        state.pids = p.to_string();
        output::info(&format!("PIDs limit updated to: {}", p));
        changed = true;
    }

    if !changed {
        bail!("No updates specified. Use --model, --mem, --cpus, or --pids.");
    }

    state.save()?;

    // Re-render docker-compose.yml if resource limits changed
    if mem.is_some() || cpus.is_some() || pids.is_some() {
        let env_path = config::instance_dir(name).join(".env");
        let vars = TemplateVars {
            image: state.image.clone(),
            container_name: format!("{}-{}", config::COMPOSE_PREFIX, name),
            mem_limit: state.mem.clone(),
            memswap_limit: state.mem.clone(),
            cpus: state.cpus.clone(),
            pids_limit: state.pids.clone(),
            port: state.port,
            env_file: env_path.to_string_lossy().to_string(),
            volume_prefix: format!("{}_{}", config::COMPOSE_PREFIX, name),
        };

        let compose_content = template::render(&vars)?;
        let compose_path = config::instance_dir(name).join("docker-compose.yml");
        fs::write(&compose_path, compose_content)?;
    }

    output::warn(&format!(
        "Restart the instance for changes to take effect: moltctrl restart {}",
        name
    ));

    Ok(())
}
