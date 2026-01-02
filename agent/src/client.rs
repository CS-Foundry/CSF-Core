use crate::collector::SystemMetrics;
use crate::config::AgentConfig;
use anyhow::Result;
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRegistration {
    pub agent_id: Uuid,
    pub name: String,
    pub hostname: String,
    pub os_type: String,
    pub os_version: String,
    pub architecture: String,
    pub agent_version: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Heartbeat {
    pub agent_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub status: String,
}

#[derive(Clone)]
pub struct ServerClient {
    client: Client,
    server_url: String,
    api_key: String,
}

impl ServerClient {
    pub fn new(config: &AgentConfig) -> Self {
        Self {
            client: Client::new(),
            server_url: config.server_url.clone(),
            api_key: config.api_key.clone(),
        }
    }

    pub async fn register(&self, registration: &AgentRegistration) -> Result<RegistrationResponse> {
        let url = format!("{}/api/agents/register", self.server_url);

        let response = self
            .client
            .post(&url)
            .header("X-API-Key", &self.api_key)
            .json(registration)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            anyhow::bail!("Registration failed: {}", response.status())
        }
    }

    pub async fn send_heartbeat(&self, heartbeat: &Heartbeat) -> Result<()> {
        let url = format!("{}/api/agents/heartbeat", self.server_url);

        let response = self
            .client
            .post(&url)
            .header("X-API-Key", &self.api_key)
            .json(heartbeat)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Heartbeat failed: {}", response.status());
        }

        Ok(())
    }

    pub async fn send_metrics(&self, metrics: &SystemMetrics) -> Result<()> {
        let url = format!("{}/api/agents/metrics", self.server_url);

        let response = self
            .client
            .post(&url)
            .header("X-API-Key", &self.api_key)
            .json(metrics)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Metrics upload failed: {}", response.status());
        }

        Ok(())
    }
}
