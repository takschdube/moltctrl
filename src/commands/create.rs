use std::fs;

use anyhow::{bail, Result};

use crate::config;
use crate::docker::{self, DockerCompose};
use crate::health;
use crate::output;
use crate::port;
use crate::provider;
use crate::state::InstanceState;
use crate::template::{self, TemplateVars};
use crate::token;
use crate::validate;

#[allow(clippy::too_many_arguments)]
pub async fn run(
    name: &str,
    flag_provider: Option<&str>,
    flag_api_key: Option<&str>,
    flag_model: Option<&str>,
    flag_port: Option<u16>,
    flag_image: Option<&str>,
    flag_mem: Option<&str>,
    flag_cpus: Option<&str>,
    flag_pids: Option<&str>,
    use_process: bool,
) -> Result<()> {
    validate::validate_name(name)?;
    config::ensure_dirs()?;

    // Check for existing instance
    if InstanceState::exists(name) {
        bail!(
            "Instance '{}' already exists. Destroy it first or choose another name.",
            name
        );
    }

    let isolation = if use_process { "process" } else { "docker" };

    // For docker mode, require docker
    if !use_process {
        docker::require_docker()?;
    }

    // Resolve provider and API key
    let resolved = provider::resolve(flag_provider, flag_api_key)?;
    let provider_name = &resolved.provider;
    let api_key = &resolved.api_key;

    // Resolve model
    let model = flag_model
        .map(|s| s.to_string())
        .unwrap_or_else(|| provider::default_model(provider_name).to_string());

    // Resolve port
    let port_num = if let Some(p) = flag_port {
        validate::validate_port(p)?;
        p
    } else {
        port::allocate_port()?
    };

    // Resolve image
    let image = flag_image.unwrap_or(config::DEFAULT_IMAGE);

    // Resource limits
    let mem = flag_mem.unwrap_or(config::DEFAULT_MEM);
    let cpus = flag_cpus.unwrap_or(config::DEFAULT_CPUS);
    let pids = flag_pids.unwrap_or(config::DEFAULT_PIDS);

    output::info(&format!("Creating instance '{}'...", name));
    output::debug(&format!(
        "Provider: {}, Model: {}, Port: {}, Image: {}",
        provider_name, model, port_num, image
    ));

    // Create instance directory
    let inst_dir = config::instance_dir(name);
    fs::create_dir_all(&inst_dir)?;

    // Generate auth token
    let auth_token = token::generate_auth_token();

    // Write .env file
    let env_file = inst_dir.join(".env");
    let mut env_content = format!(
        "# moltctrl instance: {}\n# Generated: {}\n\n# Provider\n",
        name,
        chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ")
    );

    if !api_key.is_empty() {
        if let Some(env_var) = provider::env_key(provider_name) {
            env_content.push_str(&format!("{}={}\n", env_var, api_key));
        }
    }
    env_content.push_str(&format!("OPENCLAW_PROVIDER={}\n", provider_name));
    env_content.push_str(&format!("OPENCLAW_MODEL={}\n", model));
    env_content.push_str("\n# Security\n");
    env_content.push_str("OPENCLAW_SANDBOX_MODE=all\n");
    env_content.push_str("OPENCLAW_WORKSPACE_ACCESS=none\n");
    env_content.push_str(&format!("OPENCLAW_AUTH_TOKEN={}\n", auth_token));

    fs::write(&env_file, &env_content)?;

    // Set .env file permissions to 600
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::Permissions::from_mode(0o600);
        fs::set_permissions(&env_file, perms)?;
    }

    // Save instance state
    let mut state = InstanceState::new(name, port_num, provider_name, &model, image);
    state.token = auth_token.clone();
    state.mem = mem.to_string();
    state.cpus = cpus.to_string();
    state.pids = pids.to_string();
    state.isolation = isolation.to_string();

    if use_process {
        // Process sandbox mode — just save state, don't start Docker
        state.status = "created".to_string();
        state.save()?;

        output::success(&format!("Instance '{}' created (process mode)", name));
        println!();
        output::info("Instance details:");
        println!("  Name:      {}", name);
        println!("  Provider:  {}", provider_name);
        println!("  Model:     {}", model);
        println!("  Port:      {}", port_num);
        println!("  Isolation: process");
        println!("  Token:     {}", auth_token);
        println!();
        output::info("Start the instance with: moltctrl start");
        return Ok(());
    }

    // Docker mode — render template and start containers
    let env_file_str = env_file.to_string_lossy().to_string();
    let vars = TemplateVars {
        image: image.to_string(),
        container_name: format!("{}-{}", config::COMPOSE_PREFIX, name),
        mem_limit: mem.to_string(),
        memswap_limit: mem.to_string(),
        cpus: cpus.to_string(),
        pids_limit: pids.to_string(),
        port: port_num,
        env_file: env_file_str,
        volume_prefix: format!("{}_{}", config::COMPOSE_PREFIX, name),
    };

    let compose_content = template::render(&vars)?;
    let compose_path = inst_dir.join("docker-compose.yml");
    fs::write(&compose_path, compose_content)?;

    state.save()?;

    // Pull image (streams progress directly to terminal)
    output::info(&format!("Pulling image {}...", image));
    if !DockerCompose::pull_image(image)? {
        output::warn(&format!(
            "Could not pull image '{}'. It may need to be built or may not exist yet.",
            image
        ));
    }

    // Start the container
    {
        let spinner = output::spinner("Starting instance...");
        let dc = DockerCompose::new(name, &compose_path.to_string_lossy());
        match dc.up() {
            Ok(()) => {
                spinner.finish_and_clear();
                state.status = "running".to_string();
                state.save()?;

                if health::wait_healthy(name, 120).await? {
                    output::success(&format!("Instance '{}' is healthy and running", name));
                } else {
                    output::warn(
                        "Instance started but health check hasn't passed yet. Check: moltctrl logs",
                    );
                }
            }
            Err(_) => {
                spinner.finish_and_clear();
                state.status = "error".to_string();
                state.save()?;
                bail!(
                    "Failed to start instance '{}'. Check: moltctrl logs {}",
                    name,
                    name
                );
            }
        }
    }

    println!();
    output::info("Instance details:");
    println!("  Name:     {}", name);
    println!("  Provider: {}", provider_name);
    println!("  Model:    {}", model);
    println!("  Port:     {}", port_num);
    println!("  URL:      http://127.0.0.1:{}", port_num);
    println!("  Token:    {}", auth_token);
    println!();
    output::info("Quick start:");
    println!("  moltctrl open {}     # Open in browser", name);
    println!("  moltctrl chat {}     # Chat in terminal", name);
    println!("  moltctrl logs {}     # View logs", name);

    Ok(())
}
