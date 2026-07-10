use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use ring::rand::{SecureRandom, SystemRandom};
use std::path::Path;

const PRIVATE_KEY_FILE: &str = "/var/lib/csfx-agent/wg.key";

pub struct WgIdentity {
    pub private_key_b64: String,
    pub public_key_b64: String,
}

pub fn load_or_generate() -> Result<WgIdentity> {
    if Path::new(PRIVATE_KEY_FILE).exists() {
        let private_key_b64 = std::fs::read_to_string(PRIVATE_KEY_FILE)
            .context("Failed to read WireGuard private key")?
            .trim()
            .to_string();

        let public_key_b64 = public_key_from_private(&private_key_b64)?;

        tracing::info!("WireGuard: loaded existing identity keypair");
        return Ok(WgIdentity {
            private_key_b64,
            public_key_b64,
        });
    }

    generate()
}

fn generate() -> Result<WgIdentity> {
    let rng = SystemRandom::new();
    let mut private_bytes = [0u8; 32];
    rng.fill(&mut private_bytes)
        .map_err(|_| anyhow::anyhow!("failed to generate WireGuard key"))?;
    private_bytes[0] &= 248;
    private_bytes[31] &= 127;
    private_bytes[31] |= 64;

    let secret = x25519_dalek::StaticSecret::from(private_bytes);
    let public = x25519_dalek::PublicKey::from(&secret);

    let private_key_b64 = B64.encode(private_bytes);
    let public_key_b64 = B64.encode(public.to_bytes());

    std::fs::write(PRIVATE_KEY_FILE, &private_key_b64)
        .context("Failed to write WireGuard private key")?;
    set_permissions_600(PRIVATE_KEY_FILE)?;

    tracing::info!("WireGuard: generated new identity keypair");

    Ok(WgIdentity {
        private_key_b64,
        public_key_b64,
    })
}

fn public_key_from_private(private_key_b64: &str) -> Result<String> {
    let bytes = B64
        .decode(private_key_b64)
        .context("Failed to decode WireGuard private key")?;
    let array: [u8; 32] = bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("WireGuard private key has invalid length"))?;

    let secret = x25519_dalek::StaticSecret::from(array);
    let public = x25519_dalek::PublicKey::from(&secret);
    Ok(B64.encode(public.to_bytes()))
}

#[cfg(unix)]
fn set_permissions_600(path: &str) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let perms = std::fs::Permissions::from_mode(0o600);
    std::fs::set_permissions(path, perms).context("Failed to set file permissions")
}

#[cfg(not(unix))]
fn set_permissions_600(_path: &str) -> Result<()> {
    Ok(())
}
