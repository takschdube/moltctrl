use std::fs;

use anyhow::{bail, Result};

use crate::config;
use crate::output;
use crate::state::InstanceState;
use crate::token;

pub fn run(name: &str, regenerate: bool) -> Result<()> {
    let mut state = InstanceState::require(name)?;

    if regenerate {
        let new_token = token::generate_auth_token();
        state.token = new_token.clone();
        state.save()?;

        // Update .env file
        let env_file = config::instance_dir(name).join(".env");
        if env_file.exists() {
            let content = fs::read_to_string(&env_file)?;
            let updated = content
                .lines()
                .map(|line| {
                    if line.starts_with("OPENCLAW_AUTH_TOKEN=") {
                        format!("OPENCLAW_AUTH_TOKEN={}", new_token)
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
                let perms = fs::Permissions::from_mode(0o600);
                fs::set_permissions(&env_file, perms)?;
            }
        }

        output::success("Token regenerated. Restart the instance to apply.");
        println!("{}", new_token);
    } else {
        if state.token.is_empty() {
            bail!("No token found for instance '{}'", name);
        }
        println!("{}", state.token);
    }

    Ok(())
}
