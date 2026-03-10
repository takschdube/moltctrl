use anyhow::{bail, Result};

/// Validate an instance name: must start with a letter, contain only
/// alphanumerics/hyphens/underscores, max 63 chars.
pub fn validate_name(name: &str) -> Result<()> {
    if name.is_empty() {
        bail!("Instance name is required");
    }
    let re = regex_lite::Regex::new(r"^[a-zA-Z][a-zA-Z0-9_-]{0,62}$").unwrap();
    if !re.is_match(name) {
        bail!(
            "Invalid instance name '{}': must start with a letter, contain only alphanumerics/hyphens/underscores, max 63 chars",
            name
        );
    }
    Ok(())
}

/// Validate a port number: must be between 1024 and 65535.
pub fn validate_port(port: u16) -> Result<()> {
    if port < 1024 {
        bail!(
            "Invalid port '{}': must be a number between 1024 and 65535",
            port
        );
    }
    Ok(())
}

const VALID_PROVIDERS: &[&str] = &[
    "anthropic",
    "openai",
    "google",
    "aws-bedrock",
    "openrouter",
    "ollama",
];

/// Validate a provider name.
pub fn validate_provider(provider: &str) -> Result<()> {
    if VALID_PROVIDERS.contains(&provider) {
        Ok(())
    } else {
        bail!(
            "Unknown provider '{}'. Valid: {}",
            provider,
            VALID_PROVIDERS.join(", ")
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_names() {
        assert!(validate_name("myagent").is_ok());
        assert!(validate_name("Agent1").is_ok());
        assert!(validate_name("my-agent").is_ok());
        assert!(validate_name("my_agent").is_ok());
        assert!(validate_name("a").is_ok());
        assert!(
            validate_name("A123456789012345678901234567890123456789012345678901234567890ab")
                .is_ok()
        );
    }

    #[test]
    fn test_invalid_names() {
        assert!(validate_name("").is_err());
        assert!(validate_name("1agent").is_err());
        assert!(validate_name("-agent").is_err());
        assert!(validate_name("_agent").is_err());
        assert!(validate_name("my agent").is_err());
        assert!(validate_name("my.agent").is_err());
        // 64 chars (1 start + 63 rest = too long)
        assert!(
            validate_name("A1234567890123456789012345678901234567890123456789012345678901234")
                .is_err()
        );
    }

    #[test]
    fn test_valid_ports() {
        assert!(validate_port(1024).is_ok());
        assert!(validate_port(8080).is_ok());
        assert!(validate_port(18789).is_ok());
        assert!(validate_port(65535).is_ok());
    }

    #[test]
    fn test_invalid_ports() {
        assert!(validate_port(0).is_err());
        assert!(validate_port(80).is_err());
        assert!(validate_port(1023).is_err());
    }

    #[test]
    fn test_valid_providers() {
        assert!(validate_provider("anthropic").is_ok());
        assert!(validate_provider("openai").is_ok());
        assert!(validate_provider("google").is_ok());
        assert!(validate_provider("aws-bedrock").is_ok());
        assert!(validate_provider("openrouter").is_ok());
        assert!(validate_provider("ollama").is_ok());
    }

    #[test]
    fn test_invalid_providers() {
        assert!(validate_provider("unknown").is_err());
        assert!(validate_provider("").is_err());
        assert!(validate_provider("Anthropic").is_err());
    }
}
