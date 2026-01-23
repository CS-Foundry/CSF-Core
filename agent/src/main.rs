mod client;
mod collector;
mod config;
mod connect;

use anyhow::Result;
use chrono::Utc;
use client::{AgentRegistration, Heartbeat, ServerClient};
use collector::MetricsCollector;
use config::AgentConfig;
use connect::{ensure_certificates, P2PConnector};
use std::time::Duration;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize rustls crypto provider
    let _ = rustls::crypto::ring::default_provider().install_default();

    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    info!("🚀 CSF Agent starting...");

    // Load or create configuration
    let config = AgentConfig::load().unwrap_or_else(|e| {
        warn!("Failed to load config: {}. Using defaults.", e);
        AgentConfig::default()
    });

    // Save config if it's new
    if config.api_key.is_empty() {
        warn!("⚠️  No API key configured. Agent will not be able to connect to server.");
        warn!("   Please configure the agent by editing: /etc/csf-agent/config.toml");
        config.save()?;
        return Ok(());
    }

    config.save()?;
    info!("📝 Configuration loaded");
    info!("   Agent ID: {}", config.agent_id);
    info!("   Name: {}", config.name);
    if !config.p2p_only_mode {
        info!("   Server: {}", config.server_url);
    } else {
        info!("   Mode: P2P Only (no backend connection)");
    }

    // Initialize P2P if enabled
    let p2p_connector = if config.p2p.enabled {
        info!("🔐 P2P connections enabled");

        // Get certificate directory
        let cert_dir = if let Some(parent) = std::path::Path::new(&config.p2p.cert_path).parent() {
            parent.to_path_buf()
        } else {
            std::path::PathBuf::from(if cfg!(target_os = "windows") {
                "C:\\ProgramData\\csf-agent\\certs"
            } else {
                "/etc/csf-agent/certs"
            })
        };

        // Ensure certificates exist
        match ensure_certificates(&config.name, &cert_dir, config.p2p.auto_generate_certs) {
            Ok(_) => info!("✅ Certificates ready"),
            Err(e) => {
                error!("❌ Failed to setup certificates: {}", e);
                return Err(e);
            }
        }

        // Create P2P connector
        match P2PConnector::new(
            config.agent_id,
            config.name.clone(),
            config.p2p.listen_port,
            std::path::Path::new(&config.p2p.cert_path),
            std::path::Path::new(&config.p2p.key_path),
            std::path::Path::new(&config.p2p.ca_cert_path),
        ) {
            Ok(connector) => {
                info!(
                    "✅ P2P connector initialized on port {}",
                    config.p2p.listen_port
                );
                Some(connector)
            }
            Err(e) => {
                error!("❌ Failed to create P2P connector: {}", e);
                return Err(e);
            }
        }
    } else {
        info!("ℹ️  P2P connections disabled");
        None
    };

    // Start P2P server if enabled
    if let Some(ref connector) = p2p_connector {
        let connector_clone = connector.clone();
        tokio::spawn(async move {
            if let Err(e) = connector_clone.start_server().await {
                error!("❌ P2P server error: {}", e);
            }
        });

        // Connect to configured peers
        for peer in &config.p2p.peers {
            let connector_clone = connector.clone();
            let peer_addr = peer.clone();
            tokio::spawn(async move {
                info!("🔗 Connecting to peer: {}", peer_addr);
                match connector_clone.connect_to_peer(&peer_addr).await {
                    Ok(mut stream) => {
                        info!("✅ Connected to peer: {}", peer_addr);

                        // Keep connection alive with heartbeats
                        let mut interval = tokio::time::interval(Duration::from_secs(30));
                        loop {
                            interval.tick().await;

                            if let Err(e) = connector_clone.send_heartbeat(&mut stream).await {
                                error!(
                                    "❌ Heartbeat to {} failed: {}. Reconnecting...",
                                    peer_addr, e
                                );
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        error!("❌ Failed to connect to peer {}: {}", peer_addr, e);
                    }
                }
            });
        }
    }

    // Initialize components
    let client = ServerClient::new(&config);
    let mut collector = MetricsCollector::new();

    // Register with server (skip if P2P only mode)
    if !config.p2p_only_mode {
        info!("📡 Registering with server...");
        let registration = AgentRegistration {
            agent_id: config.agent_id,
            name: config.name.clone(),
            hostname: hostname::get()
                .ok()
                .and_then(|h| h.into_string().ok())
                .unwrap_or_else(|| "unknown".to_string()),
            os_type: std::env::consts::OS.to_string(),
            os_version: sysinfo::System::os_version().unwrap_or_else(|| "Unknown".to_string()),
            architecture: std::env::consts::ARCH.to_string(),
            agent_version: env!("CARGO_PKG_VERSION").to_string(),
            tags: config.tags.clone(),
        };

        match client.register(&registration).await {
            Ok(response) => {
                info!("✅ Registration successful: {}", response.message);
            }
            Err(e) => {
                error!("❌ Registration failed: {}", e);
                warn!("   Continuing anyway, will retry on next heartbeat...");
            }
        }

        // Spawn heartbeat task
        let heartbeat_client = client.clone();
        let heartbeat_agent_id = config.agent_id;
        let heartbeat_interval = config.heartbeat_interval;
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(heartbeat_interval));
            loop {
                interval.tick().await;

                let heartbeat = Heartbeat {
                    agent_id: heartbeat_agent_id,
                    timestamp: Utc::now(),
                    status: "online".to_string(),
                };

                if let Err(e) = heartbeat_client.send_heartbeat(&heartbeat).await {
                    error!("Failed to send heartbeat: {}", e);
                } else {
                    info!("💓 Heartbeat sent");
                }
            }
        });
    } else {
        info!("ℹ️  Backend connection disabled (P2P only mode)");
    }

    // Main metrics collection loop
    info!("📊 Starting metrics collection...");
    let mut interval = tokio::time::interval(Duration::from_secs(config.collection_interval));

    loop {
        interval.tick().await;

        // Collect metrics
        let metrics = collector.collect(config.agent_id);

        info!(
            "📈 Metrics - CPU: {:.1}% | RAM: {:.1}% | Disk: {:.1}%",
            metrics.cpu_usage_percent, metrics.memory_usage_percent, metrics.disk_usage_percent
        );

        // Send to server (skip if P2P only mode)
        if !config.p2p_only_mode {
            match client.send_metrics(&metrics).await {
                Ok(_) => {
                    info!("✅ Metrics sent to server");
                }
                Err(e) => {
                    error!("❌ Failed to send metrics: {}", e);
                }
            }
        }
    }
}
