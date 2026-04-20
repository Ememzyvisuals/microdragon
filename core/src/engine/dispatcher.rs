// microdragon-core/src/engine/dispatcher.rs
use std::sync::Arc;
use anyhow::{anyhow, Result};
use tracing::info;

use super::module_registry::ModuleRegistry;

pub struct ModuleDispatcher {
    registry: Arc<ModuleRegistry>,
}

impl ModuleDispatcher {
    pub fn new(registry: Arc<ModuleRegistry>) -> Self {
        Self { registry }
    }

    pub async fn dispatch(&self, module: &str, action: &str, payload: serde_json::Value) -> Result<serde_json::Value> {
        let module_info = self.registry.get(module)
            .ok_or_else(|| anyhow!("Module '{}' not found", module))?;

        if !module_info.enabled {
            return Err(anyhow!("Module '{}' is disabled", module));
        }

        info!("Dispatching to module '{}', action '{}'", module, action);

        // In production this would spawn subprocess or use IPC
        // For now return a structured response
        Ok(serde_json::json!({
            "module": module,
            "action": action,
            "status": "dispatched",
            "payload": payload
        }))
    }
}
