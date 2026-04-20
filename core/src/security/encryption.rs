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

// ─── Audit Log ───────────────────────────────────────────────────────────────

// microdragon-core/src/security/audit.rs (inlined here for brevity)
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::io::Write;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub action: String,
    pub detail: String,
    pub severity: AuditSeverity,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditSeverity {
    Info,
    Medium,
    High,
    Critical,
}

pub struct AuditLog {
    path: PathBuf,
}

impl AuditLog {
    pub fn new(path: &Path) -> Result<Self> {
        if let Some(p) = path.parent() { std::fs::create_dir_all(p)?; }
        Ok(Self { path: path.to_path_buf() })
    }

    pub fn log_sync(&self, event: AuditEvent) {
        if let Ok(line) = serde_json::to_string(&event) {
            if let Ok(mut f) = std::fs::OpenOptions::new()
                .create(true).append(true).open(&self.path)
            {
                let _ = writeln!(f, "{}", line);
            }
        }
    }
}

// ─── Permissions ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum Capability {
    ReadFiles,
    WriteFiles,
    ExecuteCommands,
    NetworkAccess,
    SocialMessaging,
    SystemInfo,
    CalendarAccess,
    EmailAccess,
}

pub struct PermissionSet {
    granted: Vec<Capability>,
}

impl PermissionSet {
    /// Conservative defaults — user must explicitly grant dangerous caps
    pub fn default_safe() -> Self {
        Self {
            granted: vec![
                Capability::ReadFiles,
                Capability::NetworkAccess,
                Capability::SystemInfo,
            ],
        }
    }

    pub fn grant(&mut self, cap: Capability) {
        if !self.granted.contains(&cap) {
            self.granted.push(cap);
        }
    }

    pub fn has(&self, cap: &Capability) -> bool {
        self.granted.contains(cap)
    }
}
