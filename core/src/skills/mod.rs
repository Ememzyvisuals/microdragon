// microdragon-core/src/skills/mod.rs
// Skill routing and management from the Rust side

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::collections::HashMap;
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub trusted: bool,
    pub directory: String,
}

pub struct SkillManager {
    skills_dir: PathBuf,
    loaded: HashMap<String, SkillInfo>,
}

impl SkillManager {
    pub fn new() -> Self {
        let skills_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("microdragon")
            .join("skills");

        std::fs::create_dir_all(&skills_dir).ok();
        let mut manager = Self { skills_dir, loaded: HashMap::new() };
        manager.discover_skills();
        manager
    }

    fn discover_skills(&mut self) {
        if let Ok(entries) = std::fs::read_dir(&self.skills_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_dir() {
                    let skill_md = path.join("SKILL.md");
                    if skill_md.exists() {
                        let name = path.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("")
                            .to_string();
                        self.loaded.insert(name.clone(), SkillInfo {
                            name, version: "unknown".to_string(),
                            description: "".to_string(), trusted: false,
                            directory: path.to_string_lossy().to_string(),
                        });
                    }
                }
            }
        }
        info!("SkillManager: {} skills discovered", self.loaded.len());
    }

    pub fn list(&self) -> Vec<&SkillInfo> {
        self.loaded.values().collect()
    }

    pub fn get(&self, name: &str) -> Option<&SkillInfo> {
        self.loaded.get(name)
    }

    pub fn run(&self, name: &str, task: &str) -> Result<String> {
        let skill = self.get(name)
            .ok_or_else(|| anyhow::anyhow!("Skill '{}' not found", name))?;

        // Invoke Python skill engine
        let output = std::process::Command::new("python3")
            .args(["-c", &format!(
                "import sys; sys.path.insert(0, '{}'); \
                 from engine import SkillEngine; \
                 e = SkillEngine(); r = e.run_skill('{}', '{}'); \
                 print(r.output if r.success else r.error)",
                skill.directory, name, task.replace("'", "\\'")
            )])
            .output()?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}
