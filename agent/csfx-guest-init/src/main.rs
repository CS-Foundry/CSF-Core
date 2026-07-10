use anyhow::{Context, Result};
use std::process::Stdio;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::Command;
use tokio_vsock::{VsockAddr, VsockListener, VMADDR_CID_ANY};
use tracing::{error, info, warn};

const READY_PORT: u32 = 10000;
const LOG_PORT: u32 = 10001;
const EXEC_PORT: u32 = 10002;
const ENTRYPOINT_PATH: &str = "/csfx-entrypoint";

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_target(false)
        .init();

    info!("csfx-guest-init starting");

    let log_listener = VsockListener::bind(VsockAddr::new(VMADDR_CID_ANY, LOG_PORT))
        .context("Failed to bind vsock log port")?;
    let exec_listener = VsockListener::bind(VsockAddr::new(VMADDR_CID_ANY, EXEC_PORT))
        .context("Failed to bind vsock exec port")?;

    let mut child = spawn_entrypoint().await?;
    let stdout = child.stdout.take().context("child has no stdout")?;
    let stderr = child.stderr.take().context("child has no stderr")?;

    tokio::spawn(stream_logs(log_listener, stdout, stderr));
    tokio::spawn(serve_exec(exec_listener));

    signal_ready().await;

    let status = child.wait().await.context("Failed to wait on entrypoint")?;
    info!(code = ?status.code(), "entrypoint exited");

    Ok(())
}

async fn spawn_entrypoint() -> Result<tokio::process::Child> {
    Command::new(ENTRYPOINT_PATH)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("Failed to spawn entrypoint")
}

async fn signal_ready() {
    match VsockListener::bind(VsockAddr::new(VMADDR_CID_ANY, READY_PORT)) {
        Ok(listener) => {
            if let Ok((mut stream, _)) = listener.accept().await {
                let _ = stream.write_all(b"ready\n").await;
            }
        }
        Err(e) => warn!(error = %e, "Failed to bind ready port"),
    }
}

async fn stream_logs(
    listener: VsockListener,
    mut stdout: tokio::process::ChildStdout,
    mut stderr: tokio::process::ChildStderr,
) {
    let (mut stream, _) = match listener.accept().await {
        Ok(conn) => conn,
        Err(e) => {
            error!(error = %e, "Failed to accept log connection");
            return;
        }
    };

    let mut stdout_buf = [0u8; 4096];
    let mut stderr_buf = [0u8; 4096];

    loop {
        tokio::select! {
            result = stdout.read(&mut stdout_buf) => {
                match result {
                    Ok(0) => break,
                    Ok(n) => {
                        if stream.write_all(&stdout_buf[..n]).await.is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
            result = stderr.read(&mut stderr_buf) => {
                match result {
                    Ok(0) => {}
                    Ok(n) => {
                        if stream.write_all(&stderr_buf[..n]).await.is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        }
    }
}

async fn serve_exec(listener: VsockListener) {
    loop {
        let (stream, _) = match listener.accept().await {
            Ok(conn) => conn,
            Err(e) => {
                error!(error = %e, "Failed to accept exec connection");
                continue;
            }
        };

        tokio::spawn(handle_exec_session(stream));
    }
}

async fn handle_exec_session(stream: tokio_vsock::VsockStream) {
    let mut child = match Command::new("/bin/sh")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            error!(error = %e, "Failed to spawn exec shell");
            return;
        }
    };

    let mut child_stdin = match child.stdin.take() {
        Some(s) => s,
        None => return,
    };
    let mut child_stdout = match child.stdout.take() {
        Some(s) => s,
        None => return,
    };

    let (mut read_half, mut write_half) = stream.into_split();

    let output_task = tokio::spawn(async move {
        let mut buf = [0u8; 4096];
        loop {
            match child_stdout.read(&mut buf).await {
                Ok(0) => break,
                Ok(n) => {
                    if write_half.write_all(&buf[..n]).await.is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });

    let input_task = tokio::spawn(async move {
        let mut buf = [0u8; 4096];
        loop {
            match read_half.read(&mut buf).await {
                Ok(0) => break,
                Ok(n) => {
                    if child_stdin.write_all(&buf[..n]).await.is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });

    tokio::select! {
        _ = output_task => {}
        _ = input_task => {}
    }

    let _ = child.kill().await;
}
