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

pub use task::{Task, TaskResult, TaskStatus, TaskType};
pub use executor::TaskExecutor;
pub use dispatcher::ModuleDispatcher;
pub use event_bus::{EventBus, MicrodragonEvent};
pub use module_registry::ModuleRegistry;

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use tracing::{info, error};
use dashmap::DashMap;

use crate::config::MicrodragonConfig;
use crate::brain::MicrodragonBrain;
use crate::memory::MemoryStore;

/// The main MICRODRAGON engine - orchestrates all subsystems
pub struct MicrodragonEngine {
    pub brain: Arc<MicrodragonBrain>,
    pub memory: Arc<RwLock<MemoryStore>>,
    pub dispatcher: Arc<ModuleDispatcher>,
    pub event_bus: Arc<EventBus>,
    pub registry: Arc<ModuleRegistry>,
    pub config: Arc<RwLock<MicrodragonConfig>>,
    pub active_tasks: Arc<DashMap<String, Task>>,
}

impl MicrodragonEngine {
    pub async fn new(config: MicrodragonConfig) -> Result<Self> {
        info!("Initializing MICRODRAGON Engine...");

        // Initialize memory store
        let memory = MemoryStore::new(&config.storage).await?;
        info!("✓ Memory store initialized");

        // Initialize brain
        let brain = MicrodragonBrain::new(&config).await?;
        info!("✓ Brain layer initialized (provider: {})", config.ai.active_provider);

        // Initialize event bus
        let event_bus = EventBus::new();

        // Initialize module registry
        let registry = ModuleRegistry::new(&config);

        // Initialize dispatcher
        let dispatcher = ModuleDispatcher::new(registry.clone());

        info!("✓ MICRODRAGON Engine ready");

        Ok(Self {
            brain: Arc::new(brain),
            memory: Arc::new(RwLock::new(memory)),
            dispatcher: Arc::new(dispatcher),
            event_bus: Arc::new(event_bus),
            registry: Arc::new(registry),
            config: Arc::new(RwLock::new(config)),
            active_tasks: Arc::new(DashMap::new()),
        })
    }

    /// Process a user command through the full pipeline
    pub async fn process_command(&self, input: &str) -> Result<CommandResult> {
        let task_id = uuid::Uuid::new_v4().to_string();
        info!("Processing command [{}]: {}", &task_id[..8], &input[..input.len().min(80)]);

        // Get conversation context
        let context = {
            let memory = self.memory.read().await;
            memory.get_recent_context(20).await.unwrap_or_default()
        };

        // Brain processes the request
        let brain_response = self.brain.process(input, &context).await?;

        // Store the interaction
        {
            let mut memory = self.memory.write().await;
            memory.store_interaction(input, &brain_response.content).await?;
        }

        // Emit completion event
        self.event_bus.emit(MicrodragonEvent::CommandCompleted {
            task_id: task_id.clone(),
            tokens_used: brain_response.tokens_used,
        }).await;

        Ok(CommandResult {
            task_id,
            response: brain_response.content,
            model: brain_response.model,
            provider: brain_response.provider,
            tokens_used: brain_response.tokens_used,
            latency_ms: brain_response.latency_ms,
        })
    }

    /// Process with streaming output
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
            is_healthy: config.is_configured(),
            provider: config.ai.active_provider.to_string(),
            model: config.ai.providers.get_model(&config.ai.active_provider),
            active_tasks: self.active_tasks.len(),
            memory_ok: true,
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
