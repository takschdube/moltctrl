use anyhow::Result;

use crate::output;
use crate::state::{InstanceState, PairedKey};
use crate::token;

pub fn approve(name: &str, label: Option<&str>) -> Result<()> {
    let mut state = InstanceState::require(name)?;

    let key = token::generate_pairing_key();
    let label = label
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("key-{}", chrono::Utc::now().timestamp()));

    state.paired_keys.push(PairedKey {
        key: key.clone(),
        label: label.clone(),
        created: chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
    });
    state.save()?;

    output::success("Pairing key created");
    println!("  Label: {}", label);
    println!("  Key:   {}", key);
    Ok(())
}

pub fn list(name: &str) -> Result<()> {
    let state = InstanceState::require(name)?;

    if state.paired_keys.is_empty() {
        output::info(&format!("No pairing keys for instance '{}'", name));
        return Ok(());
    }

    let header = format!("{:<20} {:<36} CREATED", "LABEL", "KEY");
    println!("{}", header);
    let sep = format!("{:<20} {:<36} -------", "-----", "---");
    println!("{}", sep);

    for pk in &state.paired_keys {
        println!("{:<20} {:<36} {}", pk.label, pk.key, pk.created);
    }

    Ok(())
}

pub fn revoke(name: &str, label: &str) -> Result<()> {
    let mut state = InstanceState::require(name)?;

    let before_count = state.paired_keys.len();
    state.paired_keys.retain(|pk| pk.label != label);
    let after_count = state.paired_keys.len();

    state.save()?;

    if before_count == after_count {
        output::warn(&format!("No pairing key with label '{}' found", label));
    } else {
        output::success(&format!("Pairing key '{}' revoked", label));
    }

    Ok(())
}
