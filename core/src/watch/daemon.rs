// microdragon-core/src/watch/daemon.rs
// Watch Daemon — polls conditions and fires actions autonomously

use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::time;
use tracing::{info, warn, debug, error};
use chrono::Utc;

use crate::engine::MicrodragonEngine;
use super::conditions::{WatchCondition, ConditionType};

pub struct WatchDaemon {
    engine: Arc<MicrodragonEngine>,
    heartbeat_secs: u64,
    check_count: u64,
}

impl WatchDaemon {
    pub fn new(engine: Arc<MicrodragonEngine>) -> Self {
        Self {
            engine,
            heartbeat_secs: 60, // Check every 60 seconds
            check_count: 0,
        }
    }

    pub async fn run(&mut self) {
        info!("Watch daemon started (heartbeat: {}s)", self.heartbeat_secs);
        let mut interval = time::interval(Duration::from_secs(self.heartbeat_secs));

        loop {
            interval.tick().await;
            self.check_count += 1;
            debug!("Watch heartbeat #{}", self.check_count);

            if let Err(e) = self.tick().await {
                error!("Watch daemon tick error: {}", e);
            }
        }
    }

    async fn tick(&self) -> Result<()> {
        // Load watch conditions from memory store
        let memory = self.engine.memory.read().await;
        let conditions = memory.get_watches().await?;
        drop(memory);

        for condition in &conditions {
            if let Err(e) = self.evaluate_condition(condition).await {
                warn!("Condition eval error [{}]: {}", condition.condition, e);
            }
        }

        // Also run scheduled tasks
        self.run_scheduled_tasks().await?;

        Ok(())
    }

    async fn evaluate_condition(&self, condition: &crate::memory::sqlite::WatchCondition) -> Result<()> {
        let cond_lower = condition.condition.to_lowercase();

        // Classify condition type
        if cond_lower.contains("price") || cond_lower.contains("stock") ||
           cond_lower.contains("drops") || cond_lower.contains("above") ||
           cond_lower.contains("below")
        {
            self.check_market_condition(condition).await?;
        } else if cond_lower.contains("email") || cond_lower.contains("inbox") {
            self.check_email_condition(condition).await?;
        } else if cond_lower.contains("time") || cond_lower.contains("at ") ||
                  cond_lower.contains("daily") || cond_lower.contains("every")
        {
            self.check_time_condition(condition).await?;
        } else {
            // Generic AI-evaluated condition
            self.check_generic_condition(condition).await?;
        }

        Ok(())
    }

    async fn check_market_condition(&self, cond: &crate::memory::sqlite::WatchCondition) -> Result<()> {
        // Parse condition: "AAPL drops below $150" or "BTC above $100000"
        let prompt = format!(
            "Evaluate this market condition: '{}'\n\
             Check current market data and determine if the condition is met.\n\
             Reply with only: TRIGGERED or NOT_TRIGGERED, then a brief explanation.",
            cond.condition
        );

        let result = self.engine.process_command(&prompt).await?;
        if result.response.to_uppercase().contains("TRIGGERED") {
            self.fire_alert(cond, &result.response).await?;
        }
        Ok(())
    }

    async fn check_email_condition(&self, cond: &crate::memory::sqlite::WatchCondition) -> Result<()> {
        debug!("Email condition check: {}", cond.condition);
        // Email checking requires email module — log as pending
        Ok(())
    }

    async fn check_time_condition(&self, cond: &crate::memory::sqlite::WatchCondition) -> Result<()> {
        let now = Utc::now();
        let hour = now.format("%H").to_string().parse::<u32>().unwrap_or(0);

        // Simple: "daily at 8am" or "every morning"
        let cond_lower = cond.condition.to_lowercase();
        let is_morning = (cond_lower.contains("morning") || cond_lower.contains("8am") ||
                          cond_lower.contains("8:00")) && hour == 8;
        let is_daily = cond_lower.contains("daily") && hour == 9; // 9am default

        if is_morning || is_daily {
            self.fire_alert(cond, "Scheduled time condition triggered").await?;
        }
        Ok(())
    }

    async fn check_generic_condition(&self, cond: &crate::memory::sqlite::WatchCondition) -> Result<()> {
        // Only check generic conditions every 5 heartbeats to save tokens
        if self.check_count % 5 != 0 { return Ok(()); }

        let prompt = format!(
            "Is this condition currently true? Answer only YES or NO:\n'{}'\n\
             If YES, briefly explain why.",
            cond.condition
        );

        let result = self.engine.process_command(&prompt).await?;
        if result.response.to_uppercase().starts_with("YES") {
            self.fire_alert(cond, &result.response).await?;
        }
        Ok(())
    }

    async fn fire_alert(&self, cond: &crate::memory::sqlite::WatchCondition,
                         details: &str) -> Result<()> {
        info!("🔔 WATCH ALERT: {} — {}", cond.condition, &details[..details.len().min(80)]);

        // Execute the associated action
        let action_prompt = format!(
            "ALERT: Watch condition triggered.\n\
             Condition: {}\n\
             Details: {}\n\
             Action to take: {}",
            cond.condition, details, cond.action
        );

        self.engine.process_command(&action_prompt).await?;
        Ok(())
    }

    async fn run_scheduled_tasks(&self) -> Result<()> {
        // This would integrate with the AutonomousScheduler
        // For now: placeholder that logs pending scheduled jobs
        debug!("Checking scheduled tasks...");
        Ok(())
    }
}
