// microdragon-core/src/security/mod.rs
//
// MICRODRAGON Security Layer
// ────────────────────
// MICRODRAGON's #1 competitive edge over OpenClaw:
//   • Sandboxed execution (OpenClaw had CVE-2026-25253 RCE via WebSocket)
//   • AES-256-GCM encrypted local storage
//   • Prompt injection detection
//   • Capability-scoped permissions (no broad shell access by default)
//   • Full audit log of every action taken

pub mod sandbox;
pub mod encryption;
pub mod audit;
pub mod permissions;
pub mod prompt_guard;

pub use sandbox::ExecutionSandbox;
pub use encryption::VaultEncryption;
pub use audit::{AuditLog, AuditEvent, AuditSeverity};
pub use permissions::{PermissionSet, Capability};
pub use prompt_guard::PromptGuard;

use anyhow::Result;
use tracing::{info, warn};

/// Security coordinator — single entry point for all security checks
pub struct SecurityManager {
    pub sandbox: ExecutionSandbox,
    pub vault: VaultEncryption,
    pub audit: AuditLog,
    pub permissions: PermissionSet,
    pub guard: PromptGuard,
}

impl SecurityManager {
    pub async fn new(config: &crate::config::SecurityConfig) -> Result<Self> {
        let vault = VaultEncryption::init(&config.master_key_path)?;
        let audit = AuditLog::new(&config.master_key_path.parent()
            .unwrap_or(std::path::Path::new("."))
            .join("audit.log"))?;
        let sandbox = ExecutionSandbox::new(config.sandbox_execution);
        let permissions = PermissionSet::default_safe();
        let guard = PromptGuard::new();

        info!("Security manager initialized (sandbox={}, encryption={})",
            config.sandbox_execution, config.encryption_enabled);

        Ok(Self { sandbox, vault, audit, permissions, guard })
    }

    /// Check and sanitize a prompt before it reaches the AI
    pub fn sanitize_prompt(&self, input: &str) -> Result<String> {
        if let Some(threat) = self.guard.detect_injection(input) {
            warn!("Prompt injection attempt detected: {}", threat);
            self.audit.log_sync(AuditEvent {
                action: "PROMPT_INJECTION_BLOCKED".to_string(),
                detail: threat.clone(),
                severity: AuditSeverity::High,
                timestamp: chrono::Utc::now(),
            });
            return Err(anyhow::anyhow!("Potential prompt injection detected: {}", threat));
        }
        Ok(input.to_string())
    }
}
