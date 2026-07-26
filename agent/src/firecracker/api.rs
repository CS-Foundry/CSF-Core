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

        let response = tokio::time::timeout(
            std::time::Duration::from_secs(10),
            read_http_response(&mut stream),
        )
        .await
        .context("Timed out waiting for Firecracker API response")??;

        let response_text = String::from_utf8_lossy(&response);
        let status_line = response_text
            .lines()
            .next()
            .context("Empty Firecracker API response")?;

        if !status_line.contains("204") && !status_line.contains("200") {
            let header_end = find_header_end(&response).unwrap_or(response.len());
            let body_text = String::from_utf8_lossy(&response[header_end..]);
            anyhow::bail!(
                "Firecracker API request failed path={} response={} body={}",
                path,
                status_line,
                body_text.trim()
            );
        }

        Ok(())
    }
}

async fn read_http_response(stream: &mut UnixStream) -> Result<Vec<u8>> {
    let mut buf = Vec::new();
    let mut chunk = [0u8; 4096];

    let header_end = loop {
        if let Some(pos) = find_header_end(&buf) {
            break pos;
        }
        let n = stream
            .read(&mut chunk)
            .await
            .context("Failed to read Firecracker API response headers")?;
        if n == 0 {
            anyhow::bail!("Firecracker API closed connection before sending headers");
        }
        buf.extend_from_slice(&chunk[..n]);
    };

    let headers_text = String::from_utf8_lossy(&buf[..header_end]);
    let content_length: usize = headers_text
        .lines()
        .find_map(|line| {
            line.to_ascii_lowercase()
                .strip_prefix("content-length:")
                .map(|v| v.trim().to_string())
        })
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);

    let body_start = header_end + 4;
    while buf.len() < body_start + content_length {
        let n = stream
            .read(&mut chunk)
            .await
            .context("Failed to read Firecracker API response body")?;
        if n == 0 {
            anyhow::bail!("Firecracker API closed connection before sending full body");
        }
        buf.extend_from_slice(&chunk[..n]);
    }

    Ok(buf)
}

fn find_header_end(buf: &[u8]) -> Option<usize> {
    buf.windows(4).position(|w| w == b"\r\n\r\n")
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
