// microdragon-core/src/engine/event_bus.rs
use tokio::sync::broadcast;
use tracing::debug;

#[derive(Debug, Clone)]
pub enum MicrodragonEvent {
    CommandReceived { input: String },
    CommandCompleted { task_id: String, tokens_used: u32 },
    TaskStarted { task_id: String },
    TaskCompleted { task_id: String },
    TaskFailed { task_id: String, error: String },
    ModuleActivated { module: String },
    SocialMessageReceived { platform: String, from: String, content: String },
    SystemAlert { message: String, severity: AlertSeverity },
}

#[derive(Debug, Clone)]
pub enum AlertSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

pub struct EventBus {
    tx: broadcast::Sender<MicrodragonEvent>,
}

impl EventBus {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(1024);
        Self { tx }
    }

    pub async fn emit(&self, event: MicrodragonEvent) {
        debug!("Event emitted: {:?}", event);
        self.tx.send(event).ok();
    }

    pub fn subscribe(&self) -> broadcast::Receiver<MicrodragonEvent> {
        self.tx.subscribe()
    }
}
