// microdragon-core/src/brain/model_router.rs
// ModelRouter - Unified interface to all AI providers

use anyhow::{anyhow, Context, Result};
use reqwest::{Client, header};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::{Duration, Instant};
use tokio::sync::mpsc::Sender;
use tracing::{debug, info, warn};

use crate::config::AiConfig;
use crate::config::providers::{
    ChatMessage, CompletionRequest, CompletionResponse, MessageRole, ModelProvider,
};

pub struct ModelRouter {
    client: Client,
    config: AiConfig,
}

impl ModelRouter {
    pub async fn new(config: &AiConfig) -> Result<Self> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::USER_AGENT,
            header::HeaderValue::from_static("microdragon-agent/0.1.0"),
        );

        let client = Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(config.timeout_secs))
            .connection_verbose(false)
            .tcp_keepalive(Duration::from_secs(30))
            .build()
            .context("Failed to build HTTP client")?;

        info!("ModelRouter initialized with provider: {}", config.active_provider);
        Ok(Self { client, config: config.clone() })
    }

    pub async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        let start = Instant::now();
        let provider = &self.config.active_provider;

        debug!("Routing request to provider: {}", provider);

        let mut last_error = None;
        for attempt in 0..=self.config.max_retries {
            if attempt > 0 {
                let backoff = Duration::from_millis(500 * 2u64.pow(attempt - 1));
                warn!("Retry {} after {:?}", attempt, backoff);
                tokio::time::sleep(backoff).await;
            }

            match self.call_provider(provider, &request).await {
                Ok(mut resp) => {
                    resp.latency_ms = start.elapsed().as_millis() as u64;
                    return Ok(resp);
                }
                Err(e) => {
                    warn!("Attempt {} failed: {}", attempt + 1, e);
                    last_error = Some(e);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow!("Unknown error")))
    }

    pub async fn complete_streaming(
        &self,
        request: CompletionRequest,
        tx: Sender<String>,
    ) -> Result<()> {
        match &self.config.active_provider {
            ModelProvider::Anthropic => self.stream_anthropic(request, tx).await,
            ModelProvider::OpenAI | ModelProvider::Groq | ModelProvider::OpenRouter => {
                self.stream_openai_compatible(request, tx).await
            }
            ModelProvider::Custom => {
                // Fallback: non-streaming
                let resp = self.complete(request).await?;
                tx.send(resp.content).await.ok();
                Ok(())
            }
        }
    }

    async fn call_provider(
        &self,
        provider: &ModelProvider,
        request: &CompletionRequest,
    ) -> Result<CompletionResponse> {
        match provider {
            ModelProvider::Anthropic => self.call_anthropic(request).await,
            ModelProvider::OpenAI => self.call_openai(request).await,
            ModelProvider::Groq => self.call_groq(request).await,
            ModelProvider::OpenRouter => self.call_openrouter(request).await,
            ModelProvider::Custom => self.call_custom(request).await,
        }
    }

    // ─── Anthropic ───────────────────────────────────────────────────────────

    async fn call_anthropic(&self, request: &CompletionRequest) -> Result<CompletionResponse> {
        let api_key = self.config.providers.get_api_key(&ModelProvider::Anthropic)
            .ok_or_else(|| anyhow!("Anthropic API key not configured"))?;
        let model = self.config.providers.get_model(&ModelProvider::Anthropic);

        // Separate system message from conversation
        let (system_msg, chat_messages) = Self::split_system_messages(&request.messages);

        let body = json!({
            "model": model,
            "max_tokens": request.max_tokens.unwrap_or(4096),
            "temperature": request.temperature.unwrap_or(0.7),
            "system": system_msg,
            "messages": chat_messages.iter().map(|m| json!({
                "role": m.role.to_string(),
                "content": m.content
            })).collect::<Vec<_>>()
        });

        let resp = self.client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .context("Failed to call Anthropic API")?;

        let status = resp.status();
        let json: Value = resp.json().await.context("Failed to parse Anthropic response")?;

        if !status.is_success() {
            let err = json["error"]["message"].as_str().unwrap_or("Unknown error");
            return Err(anyhow!("Anthropic API error {}: {}", status, err));
        }

        let content = json["content"][0]["text"]
            .as_str()
            .unwrap_or("")
            .to_string();
        let input_tokens = json["usage"]["input_tokens"].as_u64().unwrap_or(0) as u32;
        let output_tokens = json["usage"]["output_tokens"].as_u64().unwrap_or(0) as u32;

        Ok(CompletionResponse {
            content,
            model,
            provider: "anthropic".to_string(),
            input_tokens,
            output_tokens,
            finish_reason: json["stop_reason"].as_str().unwrap_or("stop").to_string(),
            latency_ms: 0,
        })
    }

    async fn stream_anthropic(&self, request: CompletionRequest, tx: Sender<String>) -> Result<()> {
        let api_key = self.config.providers.get_api_key(&ModelProvider::Anthropic)
            .ok_or_else(|| anyhow!("Anthropic API key not configured"))?
            .to_string();
        let model = self.config.providers.get_model(&ModelProvider::Anthropic);
        let (system_msg, chat_messages) = Self::split_system_messages(&request.messages);

        let body = json!({
            "model": model,
            "max_tokens": request.max_tokens.unwrap_or(4096),
            "temperature": request.temperature.unwrap_or(0.7),
            "system": system_msg,
            "stream": true,
            "messages": chat_messages.iter().map(|m| json!({
                "role": m.role.to_string(),
                "content": m.content
            })).collect::<Vec<_>>()
        });

        let resp = self.client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await?;

        use futures::StreamExt;
        let mut stream = resp.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let bytes = chunk?;
            let text = String::from_utf8_lossy(&bytes);
            for line in text.lines() {
                if let Some(data) = line.strip_prefix("data: ") {
                    if data == "[DONE]" { break; }
                    if let Ok(json) = serde_json::from_str::<Value>(data) {
                        if json["type"] == "content_block_delta" {
                            if let Some(delta) = json["delta"]["text"].as_str() {
                                tx.send(delta.to_string()).await.ok();
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    // ─── OpenAI Compatible ───────────────────────────────────────────────────

    async fn call_openai(&self, request: &CompletionRequest) -> Result<CompletionResponse> {
        let api_key = self.config.providers.get_api_key(&ModelProvider::OpenAI)
            .ok_or_else(|| anyhow!("OpenAI API key not configured"))?;
        let model = self.config.providers.get_model(&ModelProvider::OpenAI);
        let base_url = self.config.providers.openai_base_url.as_deref()
            .unwrap_or("https://api.openai.com/v1");

        self.call_openai_compatible(api_key, base_url, &model, "openai", request).await
    }

    async fn call_groq(&self, request: &CompletionRequest) -> Result<CompletionResponse> {
        let api_key = self.config.providers.get_api_key(&ModelProvider::Groq)
            .ok_or_else(|| anyhow!("Groq API key not configured"))?;
        let model = self.config.providers.get_model(&ModelProvider::Groq);

        self.call_openai_compatible(api_key, "https://api.groq.com/openai/v1", &model, "groq", request).await
    }

    async fn call_openrouter(&self, request: &CompletionRequest) -> Result<CompletionResponse> {
        let api_key = self.config.providers.get_api_key(&ModelProvider::OpenRouter)
            .ok_or_else(|| anyhow!("OpenRouter API key not configured"))?;
        let model = self.config.providers.get_model(&ModelProvider::OpenRouter);

        self.call_openai_compatible(api_key, "https://openrouter.ai/api/v1", &model, "openrouter", request).await
    }

    async fn call_custom(&self, request: &CompletionRequest) -> Result<CompletionResponse> {
        let endpoint = self.config.providers.custom_endpoint.as_deref()
            .unwrap_or("http://localhost:11434/v1");
        let model = self.config.providers.get_model(&ModelProvider::Custom);
        let api_key = self.config.providers.custom_api_key.as_deref().unwrap_or("none");

        self.call_openai_compatible(api_key, endpoint, &model, "custom", request).await
    }

    async fn call_openai_compatible(
        &self,
        api_key: &str,
        base_url: &str,
        model: &str,
        provider_name: &str,
        request: &CompletionRequest,
    ) -> Result<CompletionResponse> {
        let messages: Vec<Value> = request.messages.iter().map(|m| json!({
            "role": m.role.to_string(),
            "content": m.content
        })).collect();

        let body = json!({
            "model": model,
            "messages": messages,
            "max_tokens": request.max_tokens.unwrap_or(4096),
            "temperature": request.temperature.unwrap_or(0.7),
        });

        let url = format!("{}/chat/completions", base_url);
        let resp = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .with_context(|| format!("Failed to call {} API", provider_name))?;

        let status = resp.status();
        let json: Value = resp.json().await?;

        if !status.is_success() {
            let err = json["error"]["message"].as_str().unwrap_or("Unknown error");
            return Err(anyhow!("{} API error {}: {}", provider_name, status, err));
        }

        let content = json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();
        let input_tokens = json["usage"]["prompt_tokens"].as_u64().unwrap_or(0) as u32;
        let output_tokens = json["usage"]["completion_tokens"].as_u64().unwrap_or(0) as u32;

        Ok(CompletionResponse {
            content,
            model: model.to_string(),
            provider: provider_name.to_string(),
            input_tokens,
            output_tokens,
            finish_reason: json["choices"][0]["finish_reason"]
                .as_str().unwrap_or("stop").to_string(),
            latency_ms: 0,
        })
    }

    async fn stream_openai_compatible(
        &self,
        request: CompletionRequest,
        tx: Sender<String>,
    ) -> Result<()> {
        let (api_key, base_url, model, _provider) = match &self.config.active_provider {
            ModelProvider::OpenAI => (
                self.config.providers.get_api_key(&ModelProvider::OpenAI).unwrap_or("").to_string(),
                self.config.providers.openai_base_url.clone().unwrap_or_else(|| "https://api.openai.com/v1".to_string()),
                self.config.providers.get_model(&ModelProvider::OpenAI),
                "openai",
            ),
            ModelProvider::Groq => (
                self.config.providers.get_api_key(&ModelProvider::Groq).unwrap_or("").to_string(),
                "https://api.groq.com/openai/v1".to_string(),
                self.config.providers.get_model(&ModelProvider::Groq),
                "groq",
            ),
            ModelProvider::OpenRouter => (
                self.config.providers.get_api_key(&ModelProvider::OpenRouter).unwrap_or("").to_string(),
                "https://openrouter.ai/api/v1".to_string(),
                self.config.providers.get_model(&ModelProvider::OpenRouter),
                "openrouter",
            ),
            _ => return Err(anyhow!("Streaming not supported for this provider")),
        };

        let messages: Vec<Value> = request.messages.iter().map(|m| json!({
            "role": m.role.to_string(),
            "content": m.content
        })).collect();

        let body = json!({
            "model": model,
            "messages": messages,
            "max_tokens": request.max_tokens.unwrap_or(4096),
            "temperature": request.temperature.unwrap_or(0.7),
            "stream": true,
        });

        let resp = self.client
            .post(format!("{}/chat/completions", base_url))
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        use futures::StreamExt;
        let mut stream = resp.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let bytes = chunk?;
            let text = String::from_utf8_lossy(&bytes);
            for line in text.lines() {
                if let Some(data) = line.strip_prefix("data: ") {
                    if data == "[DONE]" { break; }
                    if let Ok(json) = serde_json::from_str::<Value>(data) {
                        if let Some(delta) = json["choices"][0]["delta"]["content"].as_str() {
                            tx.send(delta.to_string()).await.ok();
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn split_system_messages(messages: &[ChatMessage]) -> (String, Vec<&ChatMessage>) {
        let mut system = String::new();
        let mut chat = Vec::new();

        for msg in messages {
            if msg.role == MessageRole::System {
                system = msg.content.clone();
            } else {
                chat.push(msg);
            }
        }
        (system, chat)
    }
}
