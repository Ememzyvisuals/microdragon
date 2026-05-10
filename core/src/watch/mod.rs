// microdragon-core/src/watch/mod.rs
// MICRODRAGON Watch Daemon — background condition monitoring and autonomous triggers
// "microdragon watch 'alert me when AAPL drops below $150'"
// Runs heartbeat checks, monitors conditions, fires actions

pub mod conditions;
pub mod daemon;
pub mod heartbeat;

pub use daemon::WatchDaemon;

use std::sync::Arc;
use tracing::info;

use crate::engine::MicrodragonEngine;

/// Run the watch daemon as a background tokio task
pub async fn start_watch_daemon(engine: Arc<MicrodragonEngine>) {
    info!("Starting MICRODRAGON Watch Daemon...");
    let mut daemon = WatchDaemon::new(engine);
    daemon.run().await;
}
