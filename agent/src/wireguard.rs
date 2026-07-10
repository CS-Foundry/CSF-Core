use anyhow::{Context, Result};
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tracing::info;

pub struct Peer {
    pub public_key: String,
    pub endpoint: Option<String>,
    pub allowed_ips: String,
}

pub fn rg_interface_name(resource_group_id: &str) -> String {
    const FNV_OFFSET_BASIS: u32 = 0x811c9dc5;
    const FNV_PRIME: u32 = 0x01000193;

    let mut hash = FNV_OFFSET_BASIS;
    for byte in resource_group_id.as_bytes() {
        hash ^= *byte as u32;
        hash = hash.wrapping_mul(FNV_PRIME);
    }

    format!("wgrg{:08x}", hash)
}

pub async fn ensure_interface(
    resource_group_id: &str,
    private_key_b64: &str,
    listen_port: u16,
) -> Result<String> {
    let iface = rg_interface_name(resource_group_id);

    if interface_exists(&iface).await? {
        return Ok(iface);
    }

    run_ip(&["link", "add", "dev", &iface, "type", "wireguard"]).await?;
    set_private_key(&iface, private_key_b64).await?;
    run_wg(&["set", &iface, "listen-port", &listen_port.to_string()]).await?;
    run_ip(&["link", "set", "up", "dev", &iface]).await?;

    info!(resource_group_id = %resource_group_id, iface = %iface, "WireGuard interface ready");

    Ok(iface)
}

pub async fn set_route(iface: &str, cidr: &str) -> Result<()> {
    let output = Command::new("ip")
        .args(["route", "add", cidr, "dev", iface])
        .output()
        .await
        .context("failed to execute ip route add")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("File exists") {
            return Ok(());
        }
        anyhow::bail!(
            "ip route add failed cidr={} iface={} stderr={}",
            cidr,
            iface,
            stderr
        );
    }

    Ok(())
}

pub async fn set_peers(iface: &str, peers: &[Peer]) -> Result<()> {
    for peer in peers {
        let mut args = vec![
            "set".to_string(),
            iface.to_string(),
            "peer".to_string(),
            peer.public_key.clone(),
            "allowed-ips".to_string(),
            peer.allowed_ips.clone(),
        ];

        if let Some(endpoint) = &peer.endpoint {
            args.push("endpoint".to_string());
            args.push(endpoint.clone());
            args.push("persistent-keepalive".to_string());
            args.push("25".to_string());
        }

        let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        run_wg(&arg_refs).await?;
    }

    Ok(())
}

async fn interface_exists(iface: &str) -> Result<bool> {
    let output = Command::new("ip")
        .args(["link", "show", iface])
        .output()
        .await
        .context("failed to execute ip link show")?;

    Ok(output.status.success())
}

async fn set_private_key(iface: &str, private_key_b64: &str) -> Result<()> {
    let mut child = tokio::process::Command::new("wg")
        .args(["set", iface, "private-key", "/dev/stdin"])
        .stdin(std::process::Stdio::piped())
        .spawn()
        .context("failed to spawn wg set private-key")?;

    let mut stdin = child.stdin.take().context("failed to open wg stdin")?;
    stdin
        .write_all(private_key_b64.as_bytes())
        .await
        .context("failed to write private key to wg stdin")?;
    drop(stdin);

    let status = child
        .wait()
        .await
        .context("failed to wait on wg set private-key")?;

    if !status.success() {
        anyhow::bail!("wg set private-key failed iface={}", iface);
    }

    Ok(())
}

async fn run_ip(args: &[&str]) -> Result<()> {
    let output = Command::new("ip")
        .args(args)
        .output()
        .await
        .context("failed to execute ip")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("ip command failed args={:?} stderr={}", args, stderr);
    }

    Ok(())
}

async fn run_wg(args: &[&str]) -> Result<()> {
    let output = Command::new("wg")
        .args(args)
        .output()
        .await
        .context("failed to execute wg")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("wg command failed args={:?} stderr={}", args, stderr);
    }

    Ok(())
}
