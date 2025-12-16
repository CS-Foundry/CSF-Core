use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Unique agent ID (generated on first run)
    pub agent_id: Uuid,

    /// Name of this agent
    pub name: String,

    /// URL of the central server
    pub server_url: String,

    /// API key for authentication
    pub api_key: String,

    /// How often to collect metrics (seconds)
    pub collection_interval: u64,

    /// How often to send heartbeat (seconds)
    pub heartbeat_interval: u64,

    /// Tags for this agent
    pub tags: Vec<String>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            agent_id: Uuid::new_v4(),
            name: hostname::get()
                .ok()
                .and_then(|h| h.into_string().ok())
                .unwrap_or_else(|| "unknown".to_string()),
            server_url: "http://localhost:8000".to_string(),
            api_key: String::new(),
            collection_interval: 30,
            heartbeat_interval: 60,
            tags: vec![],
        }
    }
}

impl AgentConfig {
    pub fn load() -> anyhow::Result<Self> {
        // Try to load from config file, otherwise use defaults
        // Check local directory first (for testing), then system path
        let local_config = std::path::PathBuf::from("config.toml");
        let system_config = Self::config_path();

        let config_path = if local_config.exists() {
            local_config
        } else {
            system_config
        };

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            Ok(toml::from_str(&content)?)
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self) -> anyhow::Result<()> {
        // Try local directory first (for testing), then system path
        let local_config = std::path::PathBuf::from("config.toml");
        let system_config = Self::config_path();

        // If we can write to local directory, use that
        let config_path = if local_config.exists() || std::env::current_dir().is_ok() {
            local_config
        } else {
            system_config
        };

        // Create parent directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }

        let content = toml::to_string_pretty(self)?;
        std::fs::write(&config_path, content)?;

        Ok(())
    }

    fn config_path() -> std::path::PathBuf {
        if cfg!(target_os = "windows") {
            std::path::PathBuf::from("C:\\ProgramData\\csf-agent\\config.toml")
        } else {
            std::path::PathBuf::from("/etc/csf-agent/config.toml")
        }
    }
}
