#![allow(dead_code)]

use std::path::PathBuf;

use tempfile::TempDir;

/// Create a temporary MOLTCTRL_DIR for testing
pub struct TestEnv {
    pub dir: TempDir,
}

impl TestEnv {
    pub fn new() -> Self {
        let dir = TempDir::new().expect("Failed to create temp dir");
        Self { dir }
    }

    pub fn moltctrl_dir(&self) -> PathBuf {
        self.dir.path().to_path_buf()
    }

    pub fn instances_dir(&self) -> PathBuf {
        self.dir.path().join("instances")
    }

    /// Create the instances directory
    pub fn setup(&self) {
        std::fs::create_dir_all(self.instances_dir()).expect("Failed to create instances dir");
    }

    /// Create a mock instance directory with a minimal instance.json
    pub fn create_mock_instance(&self, name: &str, port: u16, provider: &str) {
        self.setup();
        let inst_dir = self.instances_dir().join(name);
        std::fs::create_dir_all(&inst_dir).expect("Failed to create instance dir");

        let state = serde_json::json!({
            "name": name,
            "port": port,
            "provider": provider,
            "model": "test-model",
            "image": "test:latest",
            "created": "2024-01-01T00:00:00Z",
            "status": "running",
            "token": "deadbeef1234567890abcdef",
            "mem": "2g",
            "cpus": "2",
            "pids": "256",
            "paired_keys": [],
            "isolation": "docker",
            "pid": null
        });

        let json_path = inst_dir.join("instance.json");
        std::fs::write(json_path, serde_json::to_string_pretty(&state).unwrap())
            .expect("Failed to write instance.json");
    }
}
