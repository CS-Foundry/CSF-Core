use anyhow::{bail, Result};
use tokio::process::Command;
use tracing::info;

pub async fn rollback() -> Result<()> {
    info!("triggering nixos-rebuild switch --rollback");

    let status = Command::new("nixos-rebuild")
        .args(["switch", "--rollback"])
        .status()
        .await?;

    if !status.success() {
        bail!("nixos-rebuild switch --rollback failed with status {}", status);
    }

    info!("rollback complete");
    Ok(())
}
