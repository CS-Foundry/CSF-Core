use anyhow::{Context, Result};
use flate2::read::GzDecoder;
use oci_client::client::{ClientConfig, ClientProtocol};
use oci_client::config::ConfigFile;
use oci_client::secrets::RegistryAuth;
use oci_client::{Client, Reference};
use std::path::{Path, PathBuf};
use tokio::process::Command;
use tracing::info;

const ROOTFS_CACHE_DIR: &str = "/var/lib/csfx-agent/rootfs";
const ROOTFS_SIZE_MB: u64 = 2048;
const GZIP_LAYER_MEDIA_TYPES: &[&str] = &[
    "application/vnd.oci.image.layer.v1.tar+gzip",
    "application/vnd.docker.image.rootfs.diff.tar.gzip",
];
const WHITEOUT_PREFIX: &str = ".wh.";
const WHITEOUT_OPAQUE_MARKER: &str = ".wh..wh..opq";

pub struct RootfsBuilder {
    client: Client,
}

impl Default for RootfsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl RootfsBuilder {
    pub fn new() -> Self {
        Self {
            client: Client::new(ClientConfig {
                protocol: ClientProtocol::Https,
                ..Default::default()
            }),
        }
    }

    pub async fn ensure_rootfs(&self, image: &str) -> Result<PathBuf> {
        let reference: Reference = image.parse().context("Invalid image reference")?;
        let auth = RegistryAuth::Anonymous;

        let (_, digest) = self
            .client
            .pull_manifest(&reference, &auth)
            .await
            .context("Failed to resolve image manifest")?;

        let cache_key = digest.trim_start_matches("sha256:");
        let image_path = Path::new(ROOTFS_CACHE_DIR).join(format!("{}.ext4", cache_key));

        if image_path.exists() {
            info!(image = %image, digest = %digest, path = ?image_path, "Rootfs cache hit");
            return Ok(image_path);
        }

        info!(image = %image, digest = %digest, "Building rootfs image");

        let image_data = self
            .client
            .pull(&reference, &auth, GZIP_LAYER_MEDIA_TYPES.to_vec())
            .await
            .context("Failed to pull image layers")?;

        let extract_dir = Path::new(ROOTFS_CACHE_DIR).join(format!("{}.extract", cache_key));
        tokio::fs::create_dir_all(&extract_dir)
            .await
            .context("Failed to create rootfs extraction directory")?;

        for layer in &image_data.layers {
            extract_layer(&layer.data, &extract_dir)
                .await
                .context("Failed to extract image layer")?;
        }

        write_entrypoint_metadata(&image_data.config.data, &extract_dir).await?;

        let tmp_image_path = Path::new(ROOTFS_CACHE_DIR).join(format!("{}.ext4.tmp", cache_key));
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
}

async fn extract_layer(gzip_data: &[u8], dest: &Path) -> Result<()> {
    let dest = dest.to_path_buf();
    let gzip_data = gzip_data.to_vec();

    tokio::task::spawn_blocking(move || extract_layer_blocking(&gzip_data, &dest))
        .await
        .context("Layer extraction task panicked")?
}

fn extract_layer_blocking(gzip_data: &[u8], dest: &Path) -> Result<()> {
    let decoder = GzDecoder::new(gzip_data);
    let mut archive = tar::Archive::new(decoder);

    let entries = archive.entries().context("Failed to read layer entries")?;
    for entry in entries {
        let mut entry = entry.context("Failed to read layer entry")?;
        let path = entry.path().context("Invalid entry path")?.into_owned();

        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name == WHITEOUT_OPAQUE_MARKER {
                clear_opaque_directory(dest, &path)?;
                continue;
            }
            if let Some(deleted) = name.strip_prefix(WHITEOUT_PREFIX) {
                apply_whiteout(dest, &path, deleted)?;
                continue;
            }
        }

        entry
            .unpack_in(dest)
            .with_context(|| format!("Failed to unpack {:?}", path))?;
    }

    Ok(())
}

fn clear_opaque_directory(dest: &Path, whiteout_path: &Path) -> Result<()> {
    let Some(parent) = whiteout_path.parent() else {
        return Ok(());
    };
    let target_dir = dest.join(parent);
    if target_dir.exists() {
        std::fs::remove_dir_all(&target_dir)
            .with_context(|| format!("Failed to clear opaque directory {:?}", target_dir))?;
        std::fs::create_dir_all(&target_dir)
            .with_context(|| format!("Failed to recreate opaque directory {:?}", target_dir))?;
    }
    Ok(())
}

fn apply_whiteout(dest: &Path, whiteout_path: &Path, deleted_name: &str) -> Result<()> {
    let parent = whiteout_path.parent().unwrap_or_else(|| Path::new(""));
    let target = dest.join(parent).join(deleted_name);

    if target.is_dir() {
        let _ = std::fs::remove_dir_all(&target);
    } else {
        let _ = std::fs::remove_file(&target);
    }

    Ok(())
}

async fn write_entrypoint_metadata(config_data: &[u8], extract_dir: &Path) -> Result<()> {
    let config_file: ConfigFile =
        serde_json::from_slice(config_data).context("Failed to parse image config")?;

    let config = config_file.config.unwrap_or_default();
    let entrypoint = serde_json::json!({
        "entrypoint": config.entrypoint,
        "cmd": config.cmd,
        "env": config.env,
        "working_dir": config.working_dir,
    });

    let csfx_dir = extract_dir.join("csfx");
    tokio::fs::create_dir_all(&csfx_dir)
        .await
        .context("Failed to create csfx metadata directory")?;

    tokio::fs::write(
        csfx_dir.join("entrypoint.json"),
        serde_json::to_vec(&entrypoint).context("Failed to serialize entrypoint metadata")?,
    )
    .await
    .context("Failed to write entrypoint metadata")?;

    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn build_gzip_layer(entries: &[(&str, &[u8], bool)]) -> Vec<u8> {
        let mut builder = tar::Builder::new(Vec::new());
        for (path, contents, is_dir) in entries {
            if *is_dir {
                let mut header = tar::Header::new_gnu();
                header.set_path(path).unwrap();
                header.set_entry_type(tar::EntryType::Directory);
                header.set_size(0);
                header.set_cksum();
                builder.append(&header, std::io::empty()).unwrap();
            } else {
                let mut header = tar::Header::new_gnu();
                header.set_path(path).unwrap();
                header.set_size(contents.len() as u64);
                header.set_cksum();
                builder.append(&header, *contents).unwrap();
            }
        }
        let tar_bytes = builder.into_inner().unwrap();

        let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
        encoder.write_all(&tar_bytes).unwrap();
        encoder.finish().unwrap()
    }

    #[test]
    fn whiteout_removes_earlier_layer_file() {
        let tmp = std::env::temp_dir().join(format!("csfx-rootfs-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(tmp.join("a")).unwrap();

        let layer1 = build_gzip_layer(&[("a/b.txt", b"hello", false)]);
        let layer2 = build_gzip_layer(&[("a/.wh.b.txt", b"", false)]);

        extract_layer_blocking(&layer1, &tmp).unwrap();
        assert!(tmp.join("a/b.txt").exists());

        extract_layer_blocking(&layer2, &tmp).unwrap();
        assert!(!tmp.join("a/b.txt").exists());

        std::fs::remove_dir_all(&tmp).unwrap();
    }
}
