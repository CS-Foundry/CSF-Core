use futures_util::StreamExt;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::Connector;
use tracing::{info, warn};
use uuid::Uuid;

use crate::pki::AgentPki;

const MIN_RECONNECT_DELAY: Duration = Duration::from_secs(1);
const MAX_RECONNECT_DELAY: Duration = Duration::from_secs(30);

fn stream_url(gateway_url: &str) -> String {
    let ws_base = if let Some(rest) = gateway_url.strip_prefix("https://") {
        format!("wss://{}", rest)
    } else if let Some(rest) = gateway_url.strip_prefix("http://") {
        format!("ws://{}", rest)
    } else {
        format!("ws://{}", gateway_url)
    };
    format!("{}/api/internal/agent-stream", ws_base)
}

pub fn spawn(gateway_url: String, api_key: String, agent_id: Uuid) -> mpsc::UnboundedReceiver<()> {
    let (tx, rx) = mpsc::unbounded_channel();
    tokio::spawn(run(gateway_url, api_key, agent_id, tx));
    rx
}

fn build_connector() -> Result<Connector, anyhow::Error> {
    let ca_pem = AgentPki::load_ca_pem()?;
    let mut root_store = rustls::RootCertStore::empty();
    for cert in rustls_pemfile::certs(&mut ca_pem.as_bytes()) {
        root_store.add(cert?)?;
    }

    let config = rustls::ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    Ok(Connector::Rustls(Arc::new(config)))
}

async fn run(gateway_url: String, api_key: String, agent_id: Uuid, tx: mpsc::UnboundedSender<()>) {
    let url = stream_url(&gateway_url);
    let mut delay = MIN_RECONNECT_DELAY;

    let connector = match build_connector() {
        Ok(c) => Some(c),
        Err(e) => {
            warn!(agent_id = %agent_id, error = %e, "failed to build agent stream tls connector, using default");
            None
        }
    };

    loop {
        let mut request = match url.clone().into_client_request() {
            Ok(r) => r,
            Err(e) => {
                warn!(agent_id = %agent_id, error = %e, "invalid agent stream url");
                return;
            }
        };
        request.headers_mut().insert(
            "X-API-Key",
            match api_key.parse() {
                Ok(v) => v,
                Err(e) => {
                    warn!(agent_id = %agent_id, error = %e, "invalid api key header value");
                    return;
                }
            },
        );

        match tokio_tungstenite::connect_async_tls_with_config(
            request,
            None,
            false,
            connector.clone(),
        )
        .await
        {
            Ok((socket, _)) => {
                info!(agent_id = %agent_id, "agent stream connected");
                delay = MIN_RECONNECT_DELAY;

                let (_sink, mut stream) = socket.split();
                while let Some(msg) = stream.next().await {
                    match msg {
                        Ok(Message::Text(_)) => {
                            let _ = tx.send(());
                        }
                        Ok(Message::Close(_)) => break,
                        Ok(_) => continue,
                        Err(e) => {
                            warn!(agent_id = %agent_id, error = %e, "agent stream read error");
                            break;
                        }
                    }
                }

                warn!(agent_id = %agent_id, "agent stream disconnected, reconnecting");
            }
            Err(e) => {
                warn!(agent_id = %agent_id, error = %e, delay_secs = delay.as_secs(), "agent stream connect failed, retrying");
            }
        }

        tokio::time::sleep(delay).await;
        delay = std::cmp::min(delay * 2, MAX_RECONNECT_DELAY);
    }
}
