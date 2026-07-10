use anyhow::{Context, Result};
use tokio::process::Command;
use tracing::info;

const TABLE_NAME: &str = "csfx";
const CHAIN_NAME: &str = "rg_isolation";

pub async fn ensure_table_and_chain() -> Result<()> {
    run_nft(&["add", "table", "inet", TABLE_NAME]).await?;

    run_nft(&[
        "add",
        "chain",
        "inet",
        TABLE_NAME,
        CHAIN_NAME,
        "{ type filter hook forward priority 0 ; policy accept ; }",
    ])
    .await?;

    info!(
        table = TABLE_NAME,
        chain = CHAIN_NAME,
        "nftables RG isolation chain ready"
    );
    Ok(())
}

pub async fn isolate_bridge(bridge_name: &str, other_bridges: &[String]) -> Result<()> {
    for other in other_bridges {
        if other == bridge_name {
            continue;
        }

        add_drop_rule(bridge_name, other).await?;
        add_drop_rule(other, bridge_name).await?;
    }

    Ok(())
}

async fn add_drop_rule(from_bridge: &str, to_bridge: &str) -> Result<()> {
    run_nft(&[
        "add",
        "rule",
        "inet",
        TABLE_NAME,
        CHAIN_NAME,
        "iifname",
        from_bridge,
        "oifname",
        to_bridge,
        "drop",
    ])
    .await
}

pub async fn remove_bridge_rules(bridge_name: &str) -> Result<()> {
    let output = Command::new("nft")
        .args(["-a", "list", "chain", "inet", TABLE_NAME, CHAIN_NAME])
        .output()
        .await
        .context("failed to list nftables rules")?;

    let listing = String::from_utf8_lossy(&output.stdout);
    let mut handles = Vec::new();

    for line in listing.lines() {
        if !line.contains(bridge_name) {
            continue;
        }
        if let Some(handle) = line
            .rsplit("handle ")
            .next()
            .and_then(|h| h.trim().parse::<u32>().ok())
        {
            handles.push(handle);
        }
    }

    for handle in handles {
        run_nft(&[
            "delete",
            "rule",
            "inet",
            TABLE_NAME,
            CHAIN_NAME,
            "handle",
            &handle.to_string(),
        ])
        .await?;
    }

    Ok(())
}

async fn run_nft(args: &[&str]) -> Result<()> {
    let output = Command::new("nft")
        .args(args)
        .output()
        .await
        .context("failed to execute nft")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("File exists") {
            return Ok(());
        }
        anyhow::bail!("nft command failed args={:?} stderr={}", args, stderr);
    }

    Ok(())
}
