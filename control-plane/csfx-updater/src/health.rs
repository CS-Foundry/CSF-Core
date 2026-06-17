use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, warn};

pub async fn wait_for_gateway(
    gateway_url: &str,
    timeout_secs: u64,
    retry_interval_secs: u64,
) -> bool {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(timeout_secs);
    let probe_url = format!("{}/api/public-key", gateway_url.trim_end_matches('/'));
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .danger_accept_invalid_certs(true)
        .build()
        .expect("failed to build http client");

    loop {
        match client.get(&probe_url).send().await {
            Ok(resp) if resp.status().is_success() || resp.status().as_u16() == 401 => {
                info!(url = %probe_url, "api-gateway health check passed");
                return true;
            }
            Ok(resp) => {
                warn!(url = %probe_url, status = %resp.status(), "api-gateway returned unexpected status");
            }
            Err(e) => {
                warn!(url = %probe_url, error = %e, "api-gateway not reachable");
            }
        }

        if tokio::time::Instant::now() >= deadline {
            return false;
        }

        sleep(Duration::from_secs(retry_interval_secs)).await;
    }
}
