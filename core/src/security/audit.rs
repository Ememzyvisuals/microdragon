// microdragon-core/src/security/audit.rs
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use tracing::debug;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AuditSeverity { Low, Medium, High, Critical }

impl std::fmt::Display for AuditSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditSeverity::Low      => write!(f, "LOW"),
            AuditSeverity::Medium   => write!(f, "MEDIUM"),
            AuditSeverity::High     => write!(f, "HIGH"),
            AuditSeverity::Critical => write!(f, "CRITICAL"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub action:    String,
    pub detail:    String,
    pub severity:  AuditSeverity,
    pub timestamp: DateTime<Utc>,
}

pub struct AuditLog {
    log_path: std::path::PathBuf,
}

impl AuditLog {
    pub fn new(log_path: &Path) -> Result<Self> {
        if let Some(parent) = log_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        Ok(Self { log_path: log_path.to_path_buf() })
    }

    pub fn log_sync(&self, event: AuditEvent) {
        let line = format!("[{}] [{}] {} — {}\n",
            event.timestamp.format("%Y-%m-%d %H:%M:%S"),
            event.severity,
            event.action,
            event.detail,
        );
        if let Ok(mut file) = OpenOptions::new()
            .create(true).append(true).open(&self.log_path)
        {
            let _ = file.write_all(line.as_bytes());
        }
        debug!("Audit: {}", line.trim());
    }
}
