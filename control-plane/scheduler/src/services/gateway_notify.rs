use uuid::Uuid;

const API_GATEWAY_URL_ENV: &str = "API_GATEWAY_URL";
const DEFAULT_API_GATEWAY_URL: &str = "http://localhost:8000";

pub async fn notify_assignment(agent_id: Uuid) {
    let base_url = std::env::var(API_GATEWAY_URL_ENV)
        .unwrap_or_else(|_| DEFAULT_API_GATEWAY_URL.to_string());
    let url = format!(
        "{}/api/internal/agents/{}/notify-assignment",
        base_url, agent_id
    );

    let client = reqwest::Client::new();
    if let Err(e) = client.post(&url).send().await {
        tracing::warn!(agent_id = %agent_id, error = %e, "failed to notify gateway of assignment");
    }
}
