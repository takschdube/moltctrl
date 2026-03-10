use std::fs;
use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::config;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairedKey {
    pub key: String,
    pub label: String,
    pub created: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceState {
    pub name: String,
    pub port: u16,
    pub provider: String,
    pub model: String,
    pub image: String,
    pub created: String,
    pub status: String,
    #[serde(default)]
    pub token: String,
    #[serde(default = "default_mem")]
    pub mem: String,
    #[serde(default = "default_cpus")]
    pub cpus: String,
    #[serde(default = "default_pids")]
    pub pids: String,
    #[serde(default)]
    pub paired_keys: Vec<PairedKey>,
    #[serde(default)]
    pub isolation: String,
    #[serde(default)]
    pub pid: Option<u32>,
}

fn default_mem() -> String {
    config::DEFAULT_MEM.to_string()
}
fn default_cpus() -> String {
    config::DEFAULT_CPUS.to_string()
}
fn default_pids() -> String {
    config::DEFAULT_PIDS.to_string()
}

impl InstanceState {
    /// Create a new instance state
    pub fn new(name: &str, port: u16, provider: &str, model: &str, image: &str) -> Self {
        let now: DateTime<Utc> = Utc::now();
        Self {
            name: name.to_string(),
            port,
            provider: provider.to_string(),
            model: model.to_string(),
            image: image.to_string(),
            created: now.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            status: "created".to_string(),
            token: String::new(),
            mem: config::DEFAULT_MEM.to_string(),
            cpus: config::DEFAULT_CPUS.to_string(),
            pids: config::DEFAULT_PIDS.to_string(),
            paired_keys: Vec::new(),
            isolation: String::new(),
            pid: None,
        }
    }

    /// Path to the instance JSON file
    fn json_path(name: &str) -> PathBuf {
        config::instance_dir(name).join("instance.json")
    }

    /// Load instance state from disk
    pub fn load(name: &str) -> Result<Self> {
        let path = Self::json_path(name);
        let data = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read instance state for '{}'", name))?;
        let state: Self = serde_json::from_str(&data)
            .with_context(|| format!("Failed to parse instance state for '{}'", name))?;
        Ok(state)
    }

    /// Save instance state to disk
    pub fn save(&self) -> Result<()> {
        let path = Self::json_path(&self.name);
        let data = serde_json::to_string_pretty(self)?;
        fs::write(&path, data)?;
        Ok(())
    }

    /// Check if an instance exists
    pub fn exists(name: &str) -> bool {
        Self::json_path(name).exists()
    }

    /// Require that an instance exists, returning an error if not
    pub fn require(name: &str) -> Result<Self> {
        if !Self::exists(name) {
            bail!(
                "Instance '{}' not found. Use 'moltctrl list' to see instances.",
                name
            );
        }
        Self::load(name)
    }
}

/// List all instance names
pub fn list_names() -> Result<Vec<String>> {
    let dir = config::instances_dir();
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut names = Vec::new();
    for entry in fs::read_dir(&dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let name = entry.file_name().to_string_lossy().to_string();
            if entry.path().join("instance.json").exists() {
                names.push(name);
            }
        }
    }
    names.sort();
    Ok(names)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instance_state_new() {
        let state = InstanceState::new(
            "test",
            18789,
            "anthropic",
            "claude-sonnet-4-20250514",
            "img:latest",
        );
        assert_eq!(state.name, "test");
        assert_eq!(state.port, 18789);
        assert_eq!(state.provider, "anthropic");
        assert_eq!(state.status, "created");
        assert!(state.paired_keys.is_empty());
    }

    #[test]
    fn test_serialization_roundtrip() {
        let state = InstanceState::new(
            "test",
            18789,
            "anthropic",
            "claude-sonnet-4-20250514",
            "img:latest",
        );
        let json = serde_json::to_string(&state).unwrap();
        let loaded: InstanceState = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.name, "test");
        assert_eq!(loaded.port, 18789);
    }
}
