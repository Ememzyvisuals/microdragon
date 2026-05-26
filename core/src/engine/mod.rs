// microdragon-core/src/engine/mod.rs
// MICRODRAGON Core Engine - Task Orchestration and Module Pipeline

pub mod task;
pub mod executor;
pub mod dispatcher;
pub mod event_bus;
pub mod module_registry;
pub mod autonomous;
pub mod agents;
pub mod pipeline;

pub use task::Task;
pub use dispatcher::ModuleDispatcher;
pub use event_bus::{EventBus, MicrodragonEvent};
pub use module_registry::ModuleRegistry;

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use tracing::info;
use dashmap::DashMap;

use crate::config::MicrodragonConfig;
use crate::brain::MicrodragonBrain;
use crate::memory::MemoryStore;
use crate::engine::pipeline::AgenticPipeline;

/// The main MICRODRAGON engine - orchestrates all subsystems
pub struct MicrodragonEngine {
    pub brain:    Arc<MicrodragonBrain>,
    pub memory:   Arc<RwLock<MemoryStore>>,
    pub dispatcher: Arc<ModuleDispatcher>,
    pub event_bus:  Arc<EventBus>,
    pub registry:   Arc<ModuleRegistry>,
    pub config:     Arc<RwLock<MicrodragonConfig>>,
    pub active_tasks: Arc<DashMap<String, Task>>,
    pipeline: Arc<AgenticPipeline>,
}

impl MicrodragonEngine {
    pub async fn new(config: MicrodragonConfig) -> Result<Self> {
        info!("Initializing MICRODRAGON Engine…");

        let memory = MemoryStore::new(&config.storage).await?;
        info!("✓ Memory store initialized");

        let brain = Arc::new(MicrodragonBrain::new(&config).await?);
        info!("✓ Brain layer initialized (provider: {})", config.ai.active_provider);

        let event_bus  = EventBus::new();
        let registry   = Arc::new(ModuleRegistry::new(&config));
        let dispatcher = ModuleDispatcher::new(Arc::clone(&registry));

        let memory_arc = Arc::new(RwLock::new(memory));
        let pipeline   = Arc::new(AgenticPipeline::new(
            Arc::clone(&brain),
            Arc::clone(&memory_arc),
        ));

        info!("✓ MICRODRAGON Engine ready — 9-phase agentic pipeline active");

        Ok(Self {
            brain,
            memory: memory_arc,
            dispatcher: Arc::new(dispatcher),
            event_bus:  Arc::new(event_bus),
            registry,
            config: Arc::new(RwLock::new(config)),
            active_tasks: Arc::new(DashMap::new()),
            pipeline,
        })
    }

    /// Process a user command through the full 9-phase agentic pipeline.
    pub async fn process_command(&self, input: &str) -> Result<CommandResult> {
        self.process_command_with_progress(input, &|_, _, _| {}).await
    }

    /// Process with a real-time progress callback.
    /// The callback is called synchronously (not across awaits) so it is Send-safe.
    pub async fn process_command_with_progress(
        &self,
        input: &str,
        progress: &(dyn Fn(u8, &'static str, &str) + Send + Sync),
    ) -> Result<CommandResult> {
        let task_id = uuid::Uuid::new_v4().to_string();
        info!("Pipeline run [{}]: {}", &task_id[..8], &input[..input.len().min(80)]);

        let context = {
            let memory = self.memory.read().await;
            memory.get_recent_context(20).await.unwrap_or_default()
        };

        let result = self.pipeline.run(input, &context, progress).await?;

        self.event_bus.emit(MicrodragonEvent::CommandCompleted {
            task_id: task_id.clone(),
            tokens_used: result.tokens_used,
        }).await;

        Ok(CommandResult {
            task_id,
            response:    result.response,
            model:       result.model,
            provider:    result.provider,
            tokens_used: result.tokens_used,
            latency_ms:  result.latency_ms,
        })
    }

    /// Stream a response (interactive mode)
    pub async fn process_streaming(
        &self,
        input: &str,
        tx: mpsc::Sender<String>,
    ) -> Result<()> {
        let context = {
            let memory = self.memory.read().await;
            memory.get_recent_context(20).await.unwrap_or_default()
        };
        self.brain.process_streaming(input, &context, tx).await
    }

    pub async fn get_config(&self) -> MicrodragonConfig {
        self.config.read().await.clone()
    }

    pub async fn update_config(&self, new_config: MicrodragonConfig) -> Result<()> {
        new_config.save()?;
        *self.config.write().await = new_config;
        Ok(())
    }

    pub async fn health_check(&self) -> EngineHealth {
        let config = self.config.read().await;
        EngineHealth {
            is_healthy:   config.is_configured(),
            provider:     config.ai.active_provider.to_string(),
            model:        config.ai.providers.get_model(&config.ai.active_provider),
            active_tasks: self.active_tasks.len(),
            memory_ok:    true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CommandResult {
    pub task_id: String,
    pub response: String,
    pub model: String,
    pub provider: String,
    pub tokens_used: u32,
    pub latency_ms: u64,
}

#[derive(Debug, Clone)]
pub struct EngineHealth {
    pub is_healthy: bool,
    pub provider: String,
    pub model: String,
    pub active_tasks: usize,
    pub memory_ok: bool,
}
