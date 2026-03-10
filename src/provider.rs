use std::io::IsTerminal;

use anyhow::{bail, Result};
use dialoguer::Select;

use crate::output;
use crate::validate;

/// Provider definition with env var mapping and default model
pub struct ProviderInfo {
    pub name: &'static str,
    pub display: &'static str,
    pub env_key: Option<&'static str>,
    pub default_model: &'static str,
}

const PROVIDERS: &[ProviderInfo] = &[
    ProviderInfo {
        name: "anthropic",
        display: "anthropic    (Claude)",
        env_key: Some("ANTHROPIC_API_KEY"),
        default_model: "claude-sonnet-4-20250514",
    },
    ProviderInfo {
        name: "openai",
        display: "openai       (GPT)",
        env_key: Some("OPENAI_API_KEY"),
        default_model: "gpt-4o",
    },
    ProviderInfo {
        name: "google",
        display: "google       (Gemini)",
        env_key: Some("GOOGLE_API_KEY"),
        default_model: "gemini-2.0-flash",
    },
    ProviderInfo {
        name: "aws-bedrock",
        display: "aws-bedrock  (AWS Bedrock)",
        env_key: Some("AWS_ACCESS_KEY_ID"),
        default_model: "anthropic.claude-sonnet-4-20250514-v1:0",
    },
    ProviderInfo {
        name: "openrouter",
        display: "openrouter   (OpenRouter)",
        env_key: Some("OPENROUTER_API_KEY"),
        default_model: "anthropic/claude-sonnet-4-20250514",
    },
    ProviderInfo {
        name: "ollama",
        display: "ollama       (Local models)",
        env_key: None,
        default_model: "llama3.1",
    },
];

/// Look up provider info by name
pub fn get_provider(name: &str) -> Option<&'static ProviderInfo> {
    PROVIDERS.iter().find(|p| p.name == name)
}

/// Get the env var key for a provider
pub fn env_key(provider: &str) -> Option<&'static str> {
    get_provider(provider).and_then(|p| p.env_key)
}

/// Get the default model for a provider
pub fn default_model(provider: &str) -> &'static str {
    get_provider(provider).map_or("", |p| p.default_model)
}

/// Resolved provider and API key
pub struct ResolvedProvider {
    pub provider: String,
    pub api_key: String,
}

/// Resolve provider from flags, env, or interactive prompt
pub fn resolve(
    flag_provider: Option<&str>,
    flag_api_key: Option<&str>,
) -> Result<ResolvedProvider> {
    // 1. Determine provider
    let provider = if let Some(p) = flag_provider {
        validate::validate_provider(p)?;
        p.to_string()
    } else if let Ok(p) = std::env::var("MOLTCTRL_PROVIDER") {
        validate::validate_provider(&p)?;
        p
    } else {
        // Interactive prompt
        if !std::io::stdin().is_terminal() {
            bail!("No provider specified. Use --provider or set MOLTCTRL_PROVIDER");
        }

        let items: Vec<&str> = PROVIDERS.iter().map(|p| p.display).collect();
        let selection = Select::new()
            .with_prompt("Select an AI provider")
            .items(&items)
            .default(0)
            .interact()?;

        PROVIDERS[selection].name.to_string()
    };

    // 2. Resolve API key
    let api_key = if let Some(key) = flag_api_key {
        key.to_string()
    } else if let Some(env_var) = env_key(&provider) {
        if let Ok(key) = std::env::var(env_var) {
            output::debug(&format!("Using API key from ${}", env_var));
            key
        } else if provider != "ollama" {
            if std::io::stdin().is_terminal() {
                let key = dialoguer::Password::new()
                    .with_prompt(format!("Enter API key for {}", provider))
                    .interact()?;
                if key.is_empty() {
                    bail!("API key is required for provider '{}'", provider);
                }
                key
            } else {
                bail!(
                    "No API key provided. Use --api-key, set ${}, or run interactively.",
                    env_var
                );
            }
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    Ok(ResolvedProvider { provider, api_key })
}
