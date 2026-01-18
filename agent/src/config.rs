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

    /// API key for authentication (optional if only P2P is used)
    pub api_key: String,

    /// Skip backend connection if only P2P mode is needed
    pub p2p_only_mode: bool,

    /// How often to collect metrics (seconds)
    pub collection_interval: u64,

    /// How often to send heartbeat (seconds)
    pub heartbeat_interval: u64,

    /// Tags for this agent
    pub tags: Vec<String>,

    /// P2P connection settings
    pub p2p: P2PConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2PConfig {
    /// Enable P2P connections between agents
    pub enabled: bool,

    /// Port to listen for P2P connections
    pub listen_port: u16,

    /// List of peer agents to connect to (host:port)
    pub peers: Vec<String>,

    /// mTLS certificate path
    pub cert_path: String,

    /// mTLS private key path
    pub key_path: String,

    /// CA certificate path for verifying peers
    pub ca_cert_path: String,

    /// Auto-generate self-signed certificates if not found
    pub auto_generate_certs: bool,
}

impl Default for P2PConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            listen_port: 8443,
            peers: vec![],
            cert_path: Self::default_cert_path().to_string_lossy().to_string(),
            key_path: Self::default_key_path().to_string_lossy().to_string(),
            ca_cert_path: Self::default_ca_cert_path().to_string_lossy().to_string(),
            auto_generate_certs: true,
        }
    }
}

impl P2PConfig {
    fn default_cert_path() -> std::path::PathBuf {
        if cfg!(target_os = "windows") {
            std::path::PathBuf::from("C:\\ProgramData\\csf-agent\\certs\\agent.crt")
        } else {
            std::path::PathBuf::from("/etc/csf-agent/certs/agent.crt")
        }
    }

    fn default_key_path() -> std::path::PathBuf {
        if cfg!(target_os = "windows") {
            std::path::PathBuf::from("C:\\ProgramData\\csf-agent\\certs\\agent.key")
        } else {
            std::path::PathBuf::from("/etc/csf-agent/certs/agent.key")
        }
    }

    fn default_ca_cert_path() -> std::path::PathBuf {
        if cfg!(target_os = "windows") {
            std::path::PathBuf::from("C:\\ProgramData\\csf-agent\\certs\\ca.crt")
        } else {
            std::path::PathBuf::from("/etc/csf-agent/certs/ca.crt")
        }
    }
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
            p2p_only_mode: false,
            collection_interval: 30,
            heartbeat_interval: 60,
            tags: vec![],
            p2p: P2PConfig::default(),
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
