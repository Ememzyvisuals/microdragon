// microdragon-core/src/watch/mod.rs
// MICRODRAGON Watch Daemon — background condition monitoring and autonomous triggers
// "microdragon watch 'alert me when AAPL drops below $150'"
// Runs heartbeat checks, monitors conditions, fires actions

pub mod conditions;
pub mod daemon;
pub mod heartbeat;

pub use daemon::WatchDaemon;
pub use conditions::{WatchCondition, ConditionType, AlertLevel};

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, debug};
use chrono::Utc;

use crate::engine::MicrodragonEngine;

/// Run the watch daemon as a background tokio task
pub async fn start_watch_daemon(engine: Arc<MicrodragonEngine>) -> tokio::task::JoinHandle<()> {
    info!("Starting MICRODRAGON Watch Daemon...");
    tokio::spawn(async move {
        let mut daemon = WatchDaemon::new(engine);
        daemon.run().await;
    })
}
