// microdragon-core/src/security/encryption.rs
// AES-256-GCM local storage encryption

use anyhow::{Context, Result};
use std::path::Path;
use rand::RngCore;

pub struct VaultEncryption {
    key: [u8; 32],
}

impl VaultEncryption {
    pub fn init(key_path: &Path) -> Result<Self> {
        let key = if key_path.exists() {
            let raw = std::fs::read(key_path)
                .context("Failed to read master key")?;
            if raw.len() != 32 {
                return Err(anyhow::anyhow!("Invalid key length"));
            }
            let mut k = [0u8; 32];
            k.copy_from_slice(&raw);
            k
        } else {
            // Generate new key
            let mut key = [0u8; 32];
            rand::thread_rng().fill_bytes(&mut key);
            if let Some(parent) = key_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(key_path, &key)?;
            // Restrict permissions on Unix
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(key_path, std::fs::Permissions::from_mode(0o600))?;
            }
            key
        };
        Ok(Self { key })
    }

    pub fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        use aes_gcm::{Aes256Gcm, KeyInit, aead::{Aead, generic_array::GenericArray}};
        let cipher = Aes256Gcm::new(GenericArray::from_slice(&self.key));
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = GenericArray::from_slice(&nonce_bytes);
        let ciphertext = cipher.encrypt(nonce, data)
            .map_err(|e| anyhow::anyhow!("Encryption failed: {:?}", e))?;
        // Prepend nonce
        let mut out = nonce_bytes.to_vec();
        out.extend(ciphertext);
        Ok(out)
    }

    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        use aes_gcm::{Aes256Gcm, KeyInit, aead::{Aead, generic_array::GenericArray}};
        if data.len() < 12 {
            return Err(anyhow::anyhow!("Data too short to decrypt"));
        }
        let (nonce_bytes, ciphertext) = data.split_at(12);
        let cipher = Aes256Gcm::new(GenericArray::from_slice(&self.key));
        let nonce = GenericArray::from_slice(nonce_bytes);
        cipher.decrypt(nonce, ciphertext)
            .map_err(|e| anyhow::anyhow!("Decryption failed: {:?}", e))
    }
}

// AuditLog, AuditSeverity, AuditEvent → see security/audit.rs
// Capability, PermissionSet           → see security/permissions.rs
