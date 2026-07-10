use anyhow::{Context, Result};
use futures_util::StreamExt;
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tracing::info;

const ROOTFS_CACHE_DIR: &str = "/var/lib/csfx-agent/rootfs";
const ROOTFS_SIZE_MB: u64 = 2048;

pub struct RootfsBuilder<'a> {
    docker: &'a bollard::Docker,
}

impl<'a> RootfsBuilder<'a> {
    pub fn new(docker: &'a bollard::Docker) -> Self {
        Self { docker }
    }

    pub async fn ensure_rootfs(&self, image: &str) -> Result<PathBuf> {
        let cache_key = image_cache_key(image);
        let image_path = Path::new(ROOTFS_CACHE_DIR).join(format!("{}.ext4", cache_key));

        if image_path.exists() {
            info!(image = %image, path = ?image_path, "Rootfs cache hit");
            return Ok(image_path);
        }

        info!(image = %image, "Building rootfs image");

        tokio::fs::create_dir_all(ROOTFS_CACHE_DIR)
            .await
            .context("Failed to create rootfs cache directory")?;

        let extract_dir = Path::new(ROOTFS_CACHE_DIR).join(format!("{}.extract", cache_key));
        tokio::fs::create_dir_all(&extract_dir)
            .await
            .context("Failed to create rootfs extraction directory")?;

        let container_id = self.create_export_container(image).await?;
        let export_result = self.export_to_directory(&container_id, &extract_dir).await;
        self.remove_container(&container_id).await;
        export_result?;

        let tmp_image_path =
            Path::new(ROOTFS_CACHE_DIR).join(format!("{}.ext4.tmp", cache_key));
        build_ext4_image(&extract_dir, &tmp_image_path).await?;

        tokio::fs::remove_dir_all(&extract_dir)
            .await
            .context("Failed to clean up rootfs extraction directory")?;

        tokio::fs::rename(&tmp_image_path, &image_path)
            .await
            .context("Failed to finalize rootfs image")?;

        info!(image = %image, path = ?image_path, "Rootfs image ready");

        Ok(image_path)
    }

    async fn create_export_container(&self, image: &str) -> Result<String> {
        use bollard::models::ContainerCreateBody;
        use bollard::query_parameters::CreateContainerOptionsBuilder;

        let config = ContainerCreateBody {
            image: Some(image.to_string()),
            ..Default::default()
        };

        let options = CreateContainerOptionsBuilder::default().build();

        let container = self
            .docker
            .create_container(Some(options), config)
            .await
            .context("Failed to create export container")?;

        Ok(container.id)
    }

    async fn remove_container(&self, container_id: &str) {
        if let Err(e) = self.docker.remove_container(container_id, None).await {
            tracing::warn!(container_id = %container_id, error = %e, "Failed to remove export container");
        }
    }

    async fn export_to_directory(&self, container_id: &str, dest: &Path) -> Result<()> {
        let mut child = Command::new("tar")
            .args(["-x", "-C"])
            .arg(dest)
            .stdin(std::process::Stdio::piped())
            .spawn()
            .context("Failed to spawn tar extraction process")?;

        let mut stdin = child.stdin.take().context("Failed to open tar stdin")?;
        let mut stream = self.docker.export_container(container_id);

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.context("Failed to read container export stream")?;
            stdin
                .write_all(&chunk)
                .await
                .context("Failed to write to tar stdin")?;
        }
        drop(stdin);

        let status = child
            .wait()
            .await
            .context("Failed to wait on tar extraction process")?;

        if !status.success() {
            anyhow::bail!("tar extraction failed for container {}", container_id);
        }

        Ok(())
    }
}

async fn build_ext4_image(source_dir: &Path, image_path: &Path) -> Result<()> {
    let size_arg = format!("{}M", ROOTFS_SIZE_MB);

    let status = Command::new("truncate")
        .args(["-s", &size_arg])
        .arg(image_path)
        .status()
        .await
        .context("Failed to execute truncate")?;

    if !status.success() {
        anyhow::bail!("truncate failed for rootfs image {:?}", image_path);
    }

    let status = Command::new("mkfs.ext4")
        .args(["-F", "-d"])
        .arg(source_dir)
        .arg(image_path)
        .status()
        .await
        .context("Failed to execute mkfs.ext4")?;

    if !status.success() {
        anyhow::bail!("mkfs.ext4 failed for rootfs image {:?}", image_path);
    }

    Ok(())
}

fn image_cache_key(image: &str) -> String {
    const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;

    let mut hash = FNV_OFFSET_BASIS;
    for byte in image.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }

    format!("{:016x}", hash)
}
