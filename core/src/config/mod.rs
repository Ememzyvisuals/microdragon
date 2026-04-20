// microdragon-core/src/config/mod.rs
// Configuration management for MICRODRAGON

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::fs;
use tracing::{info, warn};

pub mod providers;
pub use providers::{ModelProvider, ProviderConfig};

/// Root MICRODRAGON configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicrodragonConfig {
    pub version: String,
    pub ai: AiConfig,
    pub storage: StorageConfig,
    pub security: SecurityConfig,
    pub modules: ModuleConfig,
    pub social: SocialConfig,
    pub logging: LoggingConfig,
    pub performance: PerformanceConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    pub active_provider: ModelProvider,
    pub providers: ProviderConfig,
    pub max_context_tokens: usize,
    pub temperature: f64,
    pub max_retries: u32,
    pub timeout_secs: u64,
    pub system_prompt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub base_dir: PathBuf,
    pub database_path: PathBuf,
    pub vector_db_path: PathBuf,
    pub media_path: PathBuf,
    pub code_path: PathBuf,
    pub logs_path: PathBuf,
    pub encrypt_at_rest: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub encryption_enabled: bool,
    pub master_key_path: PathBuf,
    pub session_timeout_mins: u64,
    pub audit_log_enabled: bool,
    pub sandbox_execution: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleConfig {
    pub coding_enabled: bool,
    pub research_enabled: bool,
    pub social_enabled: bool,
    pub business_enabled: bool,
    pub design_enabled: bool,
    pub simulation_enabled: bool,
    pub python_path: Option<String>,
    pub node_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialConfig {
    pub whatsapp_enabled: bool,
    pub telegram_enabled: bool,
    pub telegram_bot_token: Option<String>,
    pub telegram_allowed_users: Vec<i64>,
    pub discord_enabled: bool,
    pub discord_bot_token: Option<String>,
    pub discord_allowed_guilds: Vec<String>,
    pub command_prefix: String,
    pub human_typing_delay_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file_logging: bool,
    pub max_log_size_mb: u64,
    pub keep_days: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub max_concurrent_tasks: usize,
    pub thread_pool_size: usize,
    pub cache_size_mb: usize,
    pub enable_gpu: bool,
}

impl Default for MicrodragonConfig {
    fn default() -> Self {
        let base_dir = Self::default_data_dir();
        Self {
            version: "0.1.0".to_string(),
            ai: AiConfig {
                active_provider: ModelProvider::Anthropic,
                providers: ProviderConfig::default(),
                max_context_tokens: 100_000,
                temperature: 0.7,
                max_retries: 3,
                timeout_secs: 120,
                system_prompt: DEFAULT_SYSTEM_PROMPT.to_string(),
            },
            storage: StorageConfig {
                database_path: base_dir.join("microdragon.db"),
                vector_db_path: base_dir.join("vectors"),
                media_path: base_dir.join("media"),
                code_path: base_dir.join("code"),
                logs_path: base_dir.join("logs"),
                encrypt_at_rest: true,
                base_dir: base_dir.clone(),
            },
            security: SecurityConfig {
                encryption_enabled: true,
                master_key_path: base_dir.join(".master.key"),
                session_timeout_mins: 60,
                audit_log_enabled: true,
                sandbox_execution: true,
            },
            modules: ModuleConfig {
                coding_enabled: true,
                research_enabled: true,
                social_enabled: true,
                business_enabled: false,
                design_enabled: true,
                simulation_enabled: true,
                python_path: None,
                node_path: None,
            },
            social: SocialConfig {
                whatsapp_enabled: false,
                telegram_enabled: false,
                telegram_bot_token: None,
                telegram_allowed_users: vec![],
                discord_enabled: false,
                discord_bot_token: None,
                discord_allowed_guilds: vec![],
                command_prefix: "/microdragon".to_string(),
                human_typing_delay_ms: 500,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file_logging: true,
                max_log_size_mb: 100,
                keep_days: 30,
            },
            performance: PerformanceConfig {
                max_concurrent_tasks: 8,
                thread_pool_size: 4,
                cache_size_mb: 512,
                enable_gpu: false,
            },
        }
    }
}

impl MicrodragonConfig {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path();
        if config_path.exists() {
            let content = fs::read_to_string(&config_path)
                .context("Failed to read config file")?;
            let config: MicrodragonConfig = toml::from_str(&content)
                .context("Failed to parse config file")?;
            info!("Loaded config from {:?}", config_path);
            Ok(config)
        } else {
            warn!("No config found, using defaults. Run 'microdragon setup' to configure.");
            let default = Self::default();
            // Ensure directories exist
            default.ensure_dirs()?;
            Ok(default)
        }
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path();
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)
            .context("Failed to serialize config")?;
        fs::write(&config_path, content)
            .context("Failed to write config file")?;
        info!("Config saved to {:?}", config_path);
        Ok(())
    }

    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("microdragon")
            .join("config.toml")
    }

    pub fn default_data_dir() -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("microdragon")
    }

    pub fn ensure_dirs(&self) -> Result<()> {
        let dirs_to_create = [
            &self.storage.base_dir,
            &self.storage.vector_db_path,
            &self.storage.media_path,
            &self.storage.code_path,
            &self.storage.logs_path,
        ];
        for dir in &dirs_to_create {
            fs::create_dir_all(dir)
                .with_context(|| format!("Failed to create directory: {:?}", dir))?;
        }
        Ok(())
    }

    pub fn is_configured(&self) -> bool {
        match self.ai.active_provider {
            ModelProvider::Anthropic => self.ai.providers.anthropic_api_key.is_some(),
            ModelProvider::OpenAI => self.ai.providers.openai_api_key.is_some(),
            ModelProvider::Groq => self.ai.providers.groq_api_key.is_some(),
            ModelProvider::OpenRouter => self.ai.providers.openrouter_api_key.is_some(),
            ModelProvider::Custom => self.ai.providers.custom_endpoint.is_some(),
        }
    }
}

pub const DEFAULT_SYSTEM_PROMPT: &str = r#"You are MICRODRAGON, a universal human-level AI agent. You are highly capable, proactive, and thoughtful.

Your capabilities include:
- Writing, debugging, and reviewing code in any language
- Conducting research and summarizing complex topics
- Managing tasks, schedules, and projects
- Automating workflows and repetitive tasks
- Analyzing data and generating insights
- Communicating professionally across platforms
- Planning and executing multi-step complex tasks

Guidelines:
1. Always think step-by-step before acting
2. Be precise, efficient, and accurate
3. Ask for clarification when intent is ambiguous
4. Break complex tasks into manageable subtasks
5. Provide progress updates for long-running tasks
6. Always explain your reasoning transparently
7. Prioritize security and privacy in all operations

You have access to the user's local system and external services as configured. Use tools deliberately and safely."#;

pub const DEFAULT_SYSTEM_PROMPT_PLACEHOLDER: &str = DEFAULT_SYSTEM_PROMPT;
