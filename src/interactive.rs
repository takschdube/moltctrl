use std::io::IsTerminal;

use anyhow::{bail, Result};
use dialoguer::{Input, Password, Select};
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};

use crate::chat;
use crate::commands;
use crate::config;
use crate::output;
use crate::state;

/// Interactive config saved to ~/.moltctrl/config.json
#[derive(Debug, Clone, Serialize, Deserialize)]
struct InteractiveConfig {
    default_provider: String,
    default_api_key: String,
}

/// Provider entry for the interactive selection menu
struct ProviderOption {
    name: &'static str,
    label: &'static str,
    description: &'static str,
    available: bool,
}

const PROVIDER_OPTIONS: &[ProviderOption] = &[
    ProviderOption {
        name: "anthropic",
        label: "Anthropic",
        description: "Claude Sonnet, Opus, Haiku",
        available: true,
    },
    ProviderOption {
        name: "openai",
        label: "OpenAI",
        description: "GPT-4o, o1, o3",
        available: true,
    },
    ProviderOption {
        name: "google",
        label: "Google",
        description: "Gemini 2.0 Flash, Pro",
        available: true,
    },
    ProviderOption {
        name: "openrouter",
        label: "OpenRouter",
        description: "Access multiple providers with one key",
        available: true,
    },
    ProviderOption {
        name: "aws-bedrock",
        label: "AWS Bedrock",
        description: "Coming soon",
        available: false,
    },
    ProviderOption {
        name: "ollama",
        label: "Ollama",
        description: "Coming soon",
        available: false,
    },
];

/// Path to the interactive config file
fn config_path() -> std::path::PathBuf {
    config::moltctrl_dir().join("config.json")
}

/// Load saved interactive config, if it exists
fn load_config() -> Option<InteractiveConfig> {
    let path = config_path();
    if !path.exists() {
        return None;
    }
    let data = std::fs::read_to_string(&path).ok()?;
    serde_json::from_str(&data).ok()
}

/// Save interactive config to disk
fn save_config(cfg: &InteractiveConfig) -> Result<()> {
    config::ensure_dirs()?;
    let path = config_path();
    let data = serde_json::to_string_pretty(cfg)?;
    std::fs::write(&path, data)?;

    // Set config file permissions to 600 (contains API key)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o600);
        std::fs::set_permissions(&path, perms)?;
    }

    Ok(())
}

/// Generate the next available agent name like "agent-1", "agent-2", etc.
fn next_agent_name() -> Result<String> {
    let existing = state::list_names().unwrap_or_default();
    let mut max_num = 0u32;
    for name in &existing {
        if let Some(suffix) = name.strip_prefix("agent-") {
            if let Ok(n) = suffix.parse::<u32>() {
                if n > max_num {
                    max_num = n;
                }
            }
        }
    }
    Ok(format!("agent-{}", max_num + 1))
}

/// Build the display items for the provider selection menu
fn provider_display_items() -> Vec<String> {
    PROVIDER_OPTIONS
        .iter()
        .map(|p| {
            if p.available {
                format!("{:<14} {}", p.label, p.description)
            } else {
                format!("{:<14} {} (not selectable)", p.label, p.description)
            }
        })
        .collect()
}

/// Prompt for provider selection, re-prompting if a "coming soon" option is chosen
fn prompt_provider() -> Result<&'static str> {
    let items = provider_display_items();
    loop {
        let selection = Select::new()
            .with_prompt("Select a provider")
            .items(&items)
            .default(0)
            .interact()?;

        let chosen = &PROVIDER_OPTIONS[selection];
        if !chosen.available {
            println!();
            output::warn(&format!(
                "{} is coming soon. Please choose another provider.",
                chosen.label
            ));
            println!();
            continue;
        }
        return Ok(chosen.name);
    }
}

/// Prompt for API key with masked input
fn prompt_api_key(provider: &str) -> Result<String> {
    let key = Password::new()
        .with_prompt(format!("Enter your {} API key", provider))
        .interact()?;

    if key.is_empty() {
        bail!("API key is required");
    }
    Ok(key)
}

/// Prompt for agent name with auto-generated default
fn prompt_agent_name() -> Result<String> {
    let default_name = next_agent_name()?;
    let name: String = Input::new()
        .with_prompt("Agent name")
        .default(default_name)
        .interact_text()?;

    Ok(name)
}

/// Print the styled agent summary after creation
fn print_agent_summary(name: &str, provider: &str, model: &str, port: u16) {
    use std::io::IsTerminal;
    let use_color = std::io::stdout().is_terminal();

    println!();
    if use_color {
        println!("  {} Agent ready!", "✓".bold().green());
    } else {
        println!("  ✓ Agent ready!");
    }
    println!();
    if use_color {
        println!("    {}     {}", "Name:".bold(), name);
        println!("    {} {}", "Provider:".bold(), provider);
        println!("    {}    {}", "Model:".bold(), model);
        println!("    {}     {}", "Port:".bold(), port);
    } else {
        println!("    Name:     {}", name);
        println!("    Provider: {}", provider);
        println!("    Model:    {}", model);
        println!("    Port:     {}", port);
    }
    println!();
}

/// Run the interactive wizard flow
pub async fn run_interactive() -> Result<()> {
    if !std::io::stdin().is_terminal() {
        bail!("Interactive mode requires a terminal. Use subcommands for scripted usage.");
    }

    output::banner();

    let (provider, api_key) = if let Some(saved) = load_config() {
        // Config exists — ask whether to reuse
        let choices = &["Use saved config", "Set up new provider"];
        let selection = Select::new()
            .with_prompt(format!(
                "Found saved config (provider: {})",
                saved.default_provider
            ))
            .items(choices)
            .default(0)
            .interact()?;

        if selection == 0 {
            (
                saved.default_provider.clone(),
                saved.default_api_key.clone(),
            )
        } else {
            println!();
            let provider = prompt_provider()?;
            println!();
            let api_key = prompt_api_key(provider)?;

            let cfg = InteractiveConfig {
                default_provider: provider.to_string(),
                default_api_key: api_key.clone(),
            };
            save_config(&cfg)?;

            (provider.to_string(), api_key)
        }
    } else {
        // No config — run full setup
        let provider = prompt_provider()?;
        println!();
        let api_key = prompt_api_key(provider)?;

        let cfg = InteractiveConfig {
            default_provider: provider.to_string(),
            default_api_key: api_key.clone(),
        };
        save_config(&cfg)?;
        output::success("Config saved");

        (provider.to_string(), api_key)
    };

    println!();
    let agent_name = prompt_agent_name()?;

    // Create the agent using existing create flow (process mode, defaults for everything else)
    println!();
    commands::create::run(
        &agent_name,
        Some(&provider),
        Some(&api_key),
        None, // default model
        None, // auto port
        None, // default image
        None, // default mem
        None, // default cpus
        None, // default pids
        true, // use_process (process sandbox mode)
    )
    .await?;

    // Start the agent
    commands::lifecycle::start(&agent_name)?;

    // Load the instance state to get port, model, and token for summary + chat
    let instance = state::InstanceState::load(&agent_name)?;

    // Print clean summary
    print_agent_summary(
        &agent_name,
        &instance.provider,
        &instance.model,
        instance.port,
    );

    // Drop into chat
    if std::io::stdout().is_terminal() {
        println!("  {}", "Connecting to chat...".dimmed());
    } else {
        println!("  Connecting to chat...");
    }
    println!();
    chat::start_chat(instance.port, &instance.token).await?;

    Ok(())
}
