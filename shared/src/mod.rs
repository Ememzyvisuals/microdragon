// microdragon-core/src/shared/mod.rs — common types shared across modules

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Universal module response envelope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleResponse {
    pub success: bool,
    pub module: String,
    pub action: String,
    pub data: serde_json::Value,
    pub error: Option<String>,
    pub duration_ms: u64,
    pub timestamp: DateTime<Utc>,
}

impl ModuleResponse {
    pub fn ok(module: &str, action: &str, data: serde_json::Value, ms: u64) -> Self {
        Self {
            success: true, module: module.to_string(), action: action.to_string(),
            data, error: None, duration_ms: ms, timestamp: Utc::now(),
        }
    }
    pub fn err(module: &str, action: &str, error: &str) -> Self {
        Self {
            success: false, module: module.to_string(), action: action.to_string(),
            data: serde_json::Value::Null, error: Some(error.to_string()),
            duration_ms: 0, timestamp: Utc::now(),
        }
    }
}

/// Pagination helper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub total: usize,
    pub page: usize,
    pub per_page: usize,
}

impl<T> Page<T> {
    pub fn new(items: Vec<T>, total: usize, page: usize, per_page: usize) -> Self {
        Self { items, total, page, per_page }
    }
    pub fn has_next(&self) -> bool { self.page * self.per_page < self.total }
}

/// Platform identifier
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Platform {
    Cli, WhatsApp, Telegram, Discord, Api
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Platform::Cli => write!(f, "cli"),
            Platform::WhatsApp => write!(f, "whatsapp"),
            Platform::Telegram => write!(f, "telegram"),
            Platform::Discord => write!(f, "discord"),
            Platform::Api => write!(f, "api"),
        }
    }
}

/// Semver version
#[derive(Debug, Clone)]
pub struct Version { pub major: u32, pub minor: u32, pub patch: u32 }

impl Version {
    pub const MICRODRAGON: Version = Version { major: 0, minor: 1, patch: 0 };
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}
