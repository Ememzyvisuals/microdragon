// microdragon-core/src/config/providers.rs
// AI Provider Configuration and Model Router

use serde::{Deserialize, Serialize};

/// Supported AI providers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ModelProvider {
    Anthropic,
    OpenAI,
    Groq,
    OpenRouter,
    Custom,
}

impl std::fmt::Display for ModelProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModelProvider::Anthropic => write!(f, "Anthropic (Claude)"),
            ModelProvider::OpenAI => write!(f, "OpenAI (GPT)"),
            ModelProvider::Groq => write!(f, "Groq"),
            ModelProvider::OpenRouter => write!(f, "OpenRouter"),
            ModelProvider::Custom => write!(f, "Custom/Local"),
        }
    }
}

impl ModelProvider {
    pub fn default_model(&self) -> &'static str {
        match self {
            ModelProvider::Anthropic => "claude-opus-4-5",
            ModelProvider::OpenAI => "gpt-4o",
            ModelProvider::Groq => "mixtral-8x7b-32768",
            ModelProvider::OpenRouter => "anthropic/claude-opus-4-5",
            ModelProvider::Custom => "local-model",
        }
    }

    pub fn api_base_url(&self) -> &'static str {
        match self {
            ModelProvider::Anthropic => "https://api.anthropic.com",
            ModelProvider::OpenAI => "https://api.openai.com",
            ModelProvider::Groq => "https://api.groq.com/openai",
            ModelProvider::OpenRouter => "https://openrouter.ai/api",
            ModelProvider::Custom => "http://localhost:11434",
        }
    }

    pub fn supports_streaming(&self) -> bool {
        !matches!(self, ModelProvider::Custom)
    }

    pub fn context_window(&self) -> usize {
        match self {
            ModelProvider::Anthropic => 200_000,
            ModelProvider::OpenAI => 128_000,
            ModelProvider::Groq => 32_768,
            ModelProvider::OpenRouter => 200_000,
            ModelProvider::Custom => 8_192,
        }
    }
}

/// API keys and endpoints for all providers
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProviderConfig {
    // Anthropic
    pub anthropic_api_key: Option<String>,
    pub anthropic_model: Option<String>,

    // OpenAI
    pub openai_api_key: Option<String>,
    pub openai_model: Option<String>,
    pub openai_base_url: Option<String>,

    // Groq
    pub groq_api_key: Option<String>,
    pub groq_model: Option<String>,

    // OpenRouter
    pub openrouter_api_key: Option<String>,
    pub openrouter_model: Option<String>,

    // Custom/Local (Ollama, LM Studio, etc.)
    pub custom_endpoint: Option<String>,
    pub custom_model: Option<String>,
    pub custom_api_key: Option<String>,
}

impl ProviderConfig {
    /// Get the API key for the active provider
    pub fn get_api_key(&self, provider: &ModelProvider) -> Option<&str> {
        match provider {
            ModelProvider::Anthropic => self.anthropic_api_key.as_deref(),
            ModelProvider::OpenAI => self.openai_api_key.as_deref(),
            ModelProvider::Groq => self.groq_api_key.as_deref(),
            ModelProvider::OpenRouter => self.openrouter_api_key.as_deref(),
            ModelProvider::Custom => self.custom_api_key.as_deref(),
        }
    }

    /// Get the model name for the active provider
    pub fn get_model(&self, provider: &ModelProvider) -> String {
        match provider {
            ModelProvider::Anthropic => self.anthropic_model.clone()
                .unwrap_or_else(|| provider.default_model().to_string()),
            ModelProvider::OpenAI => self.openai_model.clone()
                .unwrap_or_else(|| provider.default_model().to_string()),
            ModelProvider::Groq => self.groq_model.clone()
                .unwrap_or_else(|| provider.default_model().to_string()),
            ModelProvider::OpenRouter => self.openrouter_model.clone()
                .unwrap_or_else(|| provider.default_model().to_string()),
            ModelProvider::Custom => self.custom_model.clone()
                .unwrap_or_else(|| provider.default_model().to_string()),
        }
    }

    /// Load API keys from environment variables (override config)
    pub fn apply_env_overrides(&mut self) {
        if let Ok(key) = std::env::var("ANTHROPIC_API_KEY") {
            self.anthropic_api_key = Some(key);
        }
        if let Ok(key) = std::env::var("OPENAI_API_KEY") {
            self.openai_api_key = Some(key);
        }
        if let Ok(key) = std::env::var("GROQ_API_KEY") {
            self.groq_api_key = Some(key);
        }
        if let Ok(key) = std::env::var("OPENROUTER_API_KEY") {
            self.openrouter_api_key = Some(key);
        }
        if let Ok(model) = std::env::var("MICRODRAGON_MODEL") {
            match std::env::var("MICRODRAGON_PROVIDER").as_deref() {
                Ok("anthropic") => self.anthropic_model = Some(model),
                Ok("openai") => self.openai_model = Some(model),
                Ok("groq") => self.groq_model = Some(model),
                Ok("openrouter") => self.openrouter_model = Some(model),
                _ => {}
            }
        }
    }
}

/// Message structure for AI API calls
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
}

impl std::fmt::Display for MessageRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageRole::System => write!(f, "system"),
            MessageRole::User => write!(f, "user"),
            MessageRole::Assistant => write!(f, "assistant"),
        }
    }
}

/// AI completion request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    pub messages: Vec<ChatMessage>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f64>,
    pub stream: bool,
    pub task_context: Option<String>,
}

/// AI completion response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    pub content: String,
    pub model: String,
    pub provider: String,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub finish_reason: String,
    pub latency_ms: u64,
}
