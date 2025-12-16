mod client;
mod collector;
mod config;

use anyhow::Result;
use chrono::Utc;
use client::{AgentRegistration, Heartbeat, ServerClient};
use collector::MetricsCollector;
use config::AgentConfig;
use std::time::Duration;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<()> {
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
    info!("   Server: {}", config.server_url);

    // Initialize components
    let client = ServerClient::new(&config);
    let mut collector = MetricsCollector::new();

    // Register with server
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

    // Main metrics collection loop
    info!("📊 Starting metrics collection...");
    let mut interval = tokio::time::interval(Duration::from_secs(config.collection_interval));
    
    loop {
        interval.tick().await;
        
        // Collect metrics
        let metrics = collector.collect(config.agent_id);
        
        info!(
            "📈 Collected metrics - CPU: {:.1}% | RAM: {:.1}% | Disk: {:.1}%",
            metrics.cpu_usage_percent,
            metrics.memory_usage_percent,
            metrics.disk_usage_percent
        );
        
        // Send to server
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

