// microdragon-core/src/engine/module_registry.rs
use std::collections::HashMap;
use crate::config::MicrodragonConfig;

#[derive(Debug, Clone)]
pub struct ModuleInfo {
    pub name: String,
    pub enabled: bool,
    pub description: String,
    pub version: String,
    pub runtime: ModuleRuntime,
}

#[derive(Debug, Clone)]
pub enum ModuleRuntime {
    Rust,
    Python,
    Node,
    Shell,
}

pub struct ModuleRegistry {
    modules: HashMap<String, ModuleInfo>,
}

impl ModuleRegistry {
    pub fn new(config: &MicrodragonConfig) -> Self {
        let mut modules = HashMap::new();

        modules.insert("coding".to_string(), ModuleInfo {
            name: "coding".to_string(),
            enabled: config.modules.coding_enabled,
            description: "Code generation, debugging, and git integration".to_string(),
            version: "0.1.0".to_string(),
            runtime: ModuleRuntime::Python,
        });

        modules.insert("research".to_string(), ModuleInfo {
            name: "research".to_string(),
            enabled: config.modules.research_enabled,
            description: "Web research, crawling, and summarization".to_string(),
            version: "0.1.0".to_string(),
            runtime: ModuleRuntime::Python,
        });

        modules.insert("social".to_string(), ModuleInfo {
            name: "social".to_string(),
            enabled: config.modules.social_enabled,
            description: "WhatsApp, Telegram, Discord integration".to_string(),
            version: "0.1.0".to_string(),
            runtime: ModuleRuntime::Node,
        });

        modules.insert("business".to_string(), ModuleInfo {
            name: "business".to_string(),
            enabled: config.modules.business_enabled,
            description: "Trading, market analysis, business automation".to_string(),
            version: "0.1.0".to_string(),
            runtime: ModuleRuntime::Python,
        });

        modules.insert("design".to_string(), ModuleInfo {
            name: "design".to_string(),
            enabled: config.modules.design_enabled,
            description: "GUI automation, browser automation, design tasks".to_string(),
            version: "0.1.0".to_string(),
            runtime: ModuleRuntime::Python,
        });

        modules.insert("simulation".to_string(), ModuleInfo {
            name: "simulation".to_string(),
            enabled: config.modules.simulation_enabled,
            description: "Desktop automation, screen capture, visual AI".to_string(),
            version: "0.1.0".to_string(),
            runtime: ModuleRuntime::Python,
        });

        Self { modules }
    }

    pub fn get(&self, name: &str) -> Option<&ModuleInfo> {
        self.modules.get(name)
    }

    pub fn list_enabled(&self) -> Vec<&ModuleInfo> {
        self.modules.values().filter(|m| m.enabled).collect()
    }

    pub fn list_all(&self) -> Vec<&ModuleInfo> {
        self.modules.values().collect()
    }
}
