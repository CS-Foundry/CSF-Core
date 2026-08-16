use anyhow::{bail, Context, Result};
use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM, NONCE_LEN};
use ring::rand::{SecureRandom, SystemRandom};

pub struct SecretBox {
    key: LessSafeKey,
}

impl SecretBox {
    pub fn from_env() -> Result<Self> {
        let hex_key = std::env::var("OBJECT_STORAGE_ENCRYPTION_KEY")
            .context("OBJECT_STORAGE_ENCRYPTION_KEY must be set")?;
        let bytes = hex_decode(&hex_key).context("OBJECT_STORAGE_ENCRYPTION_KEY must be hex")?;
        if bytes.len() != 32 {
            bail!("OBJECT_STORAGE_ENCRYPTION_KEY must decode to 32 bytes");
        }
        let unbound = UnboundKey::new(&AES_256_GCM, &bytes)
            .map_err(|_| anyhow::anyhow!("failed to build encryption key"))?;
        Ok(Self {
            key: LessSafeKey::new(unbound),
        })
    }

    pub fn encrypt(&self, plaintext: &str) -> Result<Vec<u8>> {
        let rng = SystemRandom::new();
        let mut nonce_bytes = [0u8; NONCE_LEN];
        rng.fill(&mut nonce_bytes)
            .map_err(|_| anyhow::anyhow!("failed to generate nonce"))?;

        let mut in_out = plaintext.as_bytes().to_vec();
        self.key
            .seal_in_place_append_tag(
                Nonce::assume_unique_for_key(nonce_bytes),
                Aad::empty(),
                &mut in_out,
            )
            .map_err(|_| anyhow::anyhow!("encryption failed"))?;

        let mut output = nonce_bytes.to_vec();
        output.extend_from_slice(&in_out);
        Ok(output)
    }

    pub fn decrypt(&self, ciphertext: &[u8]) -> Result<String> {
        if ciphertext.len() < NONCE_LEN {
            bail!("ciphertext too short");
        }
        let (nonce_bytes, sealed) = ciphertext.split_at(NONCE_LEN);
        let mut buf = sealed.to_vec();
        let nonce = Nonce::try_assume_unique_for_key(nonce_bytes)
            .map_err(|_| anyhow::anyhow!("invalid nonce"))?;
        let plaintext = self
            .key
            .open_in_place(nonce, Aad::empty(), &mut buf)
            .map_err(|_| anyhow::anyhow!("decryption failed"))?;
        String::from_utf8(plaintext.to_vec()).context("decrypted secret is not valid utf-8")
    }
}

fn hex_decode(input: &str) -> Result<Vec<u8>> {
    if !input.len().is_multiple_of(2) {
        bail!("hex string must have even length");
    }
    (0..input.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&input[i..i + 2], 16).context("invalid hex digit"))
        .collect()
}
