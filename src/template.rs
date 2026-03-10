use anyhow::Result;

/// The embedded docker-compose template
const TEMPLATE: &str = include_str!("../profiles/openclaw.yml.tmpl");

/// Variables for template rendering
pub struct TemplateVars {
    pub image: String,
    pub container_name: String,
    pub mem_limit: String,
    pub memswap_limit: String,
    pub cpus: String,
    pub pids_limit: String,
    pub port: u16,
    pub env_file: String,
    pub volume_prefix: String,
}

/// Render the docker-compose template with the given variables
pub fn render(vars: &TemplateVars) -> Result<String> {
    let result = TEMPLATE
        .replace("${MOLTCTRL_IMAGE}", &vars.image)
        .replace("${MOLTCTRL_CONTAINER_NAME}", &vars.container_name)
        .replace("${MOLTCTRL_MEM_LIMIT}", &vars.mem_limit)
        .replace("${MOLTCTRL_MEMSWAP_LIMIT}", &vars.memswap_limit)
        .replace("${MOLTCTRL_CPUS}", &vars.cpus)
        .replace("${MOLTCTRL_PIDS_LIMIT}", &vars.pids_limit)
        .replace("${MOLTCTRL_PORT}", &vars.port.to_string())
        .replace("${MOLTCTRL_ENV_FILE}", &vars.env_file)
        .replace("${MOLTCTRL_VOLUME_PREFIX}", &vars.volume_prefix);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_template() {
        let vars = TemplateVars {
            image: "test:latest".to_string(),
            container_name: "moltctrl-test".to_string(),
            mem_limit: "2g".to_string(),
            memswap_limit: "2g".to_string(),
            cpus: "2".to_string(),
            pids_limit: "256".to_string(),
            port: 18789,
            env_file: "/tmp/.env".to_string(),
            volume_prefix: "moltctrl_test".to_string(),
        };
        let result = render(&vars).unwrap();
        assert!(result.contains("test:latest"));
        assert!(result.contains("moltctrl-test"));
        assert!(result.contains("18789:18789"));
        assert!(result.contains("moltctrl_test_data"));
        assert!(!result.contains("${MOLTCTRL_"));
    }
}
