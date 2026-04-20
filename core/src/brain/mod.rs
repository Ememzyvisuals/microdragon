// microdragon-core/src/brain/mod.rs
// MICRODRAGON Brain Layer - AI Reasoning, Planning, and Model Router

pub mod model_router;
pub mod planner;
pub mod cost_optimizer;
pub use cost_optimizer::{CostOptimizer, BudgetStatus, CostSummary};
pub mod intent;
pub mod context;

pub use model_router::ModelRouter;
pub use planner::{TaskPlanner, ExecutionPlan, PlanStep};
pub use intent::{IntentParser, ParsedIntent, IntentType};
pub use context::ContextManager;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, debug};

use crate::config::{MicrodragonConfig, AiConfig};
use crate::config::providers::{ChatMessage, CompletionRequest, CompletionResponse, MessageRole};

/// The MICRODRAGON Brain - central AI reasoning system
pub struct MicrodragonBrain {
    pub router: ModelRouter,
    pub planner: TaskPlanner,
    pub intent_parser: IntentParser,
    pub context_manager: ContextManager,
    config: Arc<RwLock<AiConfig>>,
}

impl MicrodragonBrain {
    pub async fn new(config: &MicrodragonConfig) -> Result<Self> {
        let router = ModelRouter::new(&config.ai).await?;
        let planner = TaskPlanner::new();
        let intent_parser = IntentParser::new();
        let context_manager = ContextManager::new(config.ai.max_context_tokens);

        Ok(Self {
            router,
            planner,
            intent_parser,
            context_manager,
            config: Arc::new(RwLock::new(config.ai.clone())),
        })
    }

    /// Primary reasoning entry: parse intent → plan → execute
    pub async fn process(&self, input: &str, context: &[ChatMessage]) -> Result<BrainResponse> {
        debug!("Brain processing input: {}", &input[..input.len().min(100)]);

        // 1. Parse user intent
        let intent = self.intent_parser.parse(input).await?;
        info!("Parsed intent: {:?}", intent.intent_type);

        // 2. Build execution plan for complex tasks
        let plan = if intent.requires_planning() {
            Some(self.planner.create_plan(&intent, input).await?)
        } else {
            None
        };

        // 3. Build context-aware messages
        let mut messages = self.context_manager.build_messages(context, input);

        // 4. If we have a plan, inject it
        if let Some(ref execution_plan) = plan {
            let plan_injection = format!(
                "Before responding, I'll follow this execution plan:\n{}",
                execution_plan.to_prompt()
            );
            messages.insert(messages.len() - 1, ChatMessage {
                role: MessageRole::User,
                content: plan_injection,
            });
        }

        // 5. Get AI completion
        let request = CompletionRequest {
            messages,
            max_tokens: Some(4096),
            temperature: Some(self.config.read().await.temperature),
            stream: false,
            task_context: Some(intent.intent_type.to_string()),
        };

        let response = self.router.complete(request).await?;

        Ok(BrainResponse {
            content: response.content,
            intent,
            plan,
            model: response.model,
            provider: response.provider,
            tokens_used: response.input_tokens + response.output_tokens,
            latency_ms: response.latency_ms,
        })
    }

    /// Stream a response (for interactive mode)
    pub async fn process_streaming(
        &self,
        input: &str,
        context: &[ChatMessage],
        tx: tokio::sync::mpsc::Sender<String>,
    ) -> Result<()> {
        let intent = self.intent_parser.parse(input).await?;
        let messages = self.context_manager.build_messages(context, input);

        let request = CompletionRequest {
            messages,
            max_tokens: Some(4096),
            temperature: Some(self.config.read().await.temperature),
            stream: true,
            task_context: Some(intent.intent_type.to_string()),
        };

        self.router.complete_streaming(request, tx).await
    }

    /// Quick completion without full brain pipeline (for internal use)
    pub async fn quick_complete(&self, prompt: &str) -> Result<String> {
        let config = self.config.read().await;
        let request = CompletionRequest {
            messages: vec![
                ChatMessage {
                    role: MessageRole::System,
                    content: config.system_prompt.clone(),
                },
                ChatMessage {
                    role: MessageRole::User,
                    content: prompt.to_string(),
                },
            ],
            max_tokens: Some(2048),
            temperature: Some(0.3),
            stream: false,
            task_context: None,
        };
        drop(config);

        let response = self.router.complete(request).await?;
        Ok(response.content)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrainResponse {
    pub content: String,
    pub intent: ParsedIntent,
    pub plan: Option<ExecutionPlan>,
    pub model: String,
    pub provider: String,
    pub tokens_used: u32,
    pub latency_ms: u64,
}
