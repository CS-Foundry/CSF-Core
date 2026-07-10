use anyhow::{Context, Result};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

pub struct FirecrackerApiClient {
    socket_path: String,
}

impl FirecrackerApiClient {
    pub fn new(socket_path: String) -> Self {
        Self { socket_path }
    }

    pub async fn put(&self, path: &str, body: &serde_json::Value) -> Result<()> {
        let payload = serde_json::to_vec(body).context("Failed to serialize request body")?;

        let request = format!(
            "PUT {path} HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\nContent-Length: {len}\r\nConnection: close\r\n\r\n",
            path = path,
            len = payload.len(),
        );

        let mut stream = connect_with_retry(&self.socket_path).await?;

        stream
            .write_all(request.as_bytes())
            .await
            .context("Failed to write HTTP request line")?;
        stream
            .write_all(&payload)
            .await
            .context("Failed to write HTTP request body")?;

        let mut response = Vec::new();
        stream
            .read_to_end(&mut response)
            .await
            .context("Failed to read Firecracker API response")?;

        let response_text = String::from_utf8_lossy(&response);
        let status_line = response_text
            .lines()
            .next()
            .context("Empty Firecracker API response")?;

        if !status_line.contains("204") && !status_line.contains("200") {
            anyhow::bail!(
                "Firecracker API request failed path={} response={}",
                path,
                status_line
            );
        }

        Ok(())
    }
}

async fn connect_with_retry(socket_path: &str) -> Result<UnixStream> {
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(5);

    loop {
        match UnixStream::connect(socket_path).await {
            Ok(stream) => return Ok(stream),
            Err(e) if tokio::time::Instant::now() < deadline => {
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                let _ = e;
            }
            Err(e) => {
                return Err(e).context(format!(
                    "Failed to connect to Firecracker API socket {}",
                    socket_path
                ))
            }
        }
    }
}
