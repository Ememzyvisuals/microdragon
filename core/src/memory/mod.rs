// microdragon-core/src/memory/mod.rs — unified memory: hot cache + SQLite + vector

pub mod sqlite;
pub mod vector;

use anyhow::Result;
use uuid::Uuid;
use tracing::info;

use crate::config::StorageConfig;
use crate::config::providers::{ChatMessage, MessageRole};
use self::sqlite::PersistentMemory;

pub struct MemoryStore {
    persistent: PersistentMemory,
    cache: Vec<ChatMessage>,
    pub session_id: String,
}

impl MemoryStore {
    pub async fn new(config: &StorageConfig) -> Result<Self> {
        let session_id = Uuid::new_v4().to_string();
        let persistent = PersistentMemory::open(&config.database_path, &session_id)?;
        info!("MemoryStore ready — session={}", &session_id[..8]);
        Ok(Self { persistent, cache: Vec::new(), session_id })
    }

    pub async fn store_interaction(&mut self, user_msg: &str, assistant_msg: &str) -> Result<()> {
        self.cache.push(ChatMessage { role: MessageRole::User, content: user_msg.to_string() });
        self.cache.push(ChatMessage { role: MessageRole::Assistant, content: assistant_msg.to_string() });
        if self.cache.len() > 100 { self.cache.drain(0..20); }
        self.persistent.store_message("user", user_msg, 0, "", "cli")?;
        self.persistent.store_message("assistant", assistant_msg, 0, "", "cli")?;
        Ok(())
    }

    pub async fn get_recent_context(&self, n: usize) -> Result<Vec<ChatMessage>> {
        if !self.cache.is_empty() {
            let start = self.cache.len().saturating_sub(n * 2);
            return Ok(self.cache[start..].to_vec());
        }
        self.persistent.get_recent_messages(n * 2)
    }

    pub async fn clear_context(&mut self) -> Result<()> {
        self.cache.clear();
        self.persistent.clear_session()?;
        Ok(())
    }

    pub async fn save_task(&self, id: &str, task_type: &str, input: &str,
                            output: &str, status: &str, agent: &str,
                            tokens: u32, latency_ms: u64) -> Result<()> {
        self.persistent.save_task(id, task_type, input, Some(output), status, agent, tokens, latency_ms)
    }

    pub async fn record_feedback(&self, task_id: &str, score: &str) -> Result<()> {
        self.persistent.record_feedback(task_id, score)
    }

    pub async fn get_stats(&self) -> Result<sqlite::PerformanceStats> {
        self.persistent.get_performance_stats()
    }

    pub async fn save_code(&self, filename: &str, language: &str, content: &str, description: &str) -> Result<i64> {
        self.persistent.save_code_artifact(filename, language, content, description)
    }

    pub async fn add_watch(&self, condition: &str, action: &str) -> Result<i64> {
        self.persistent.add_watch_condition(condition, action)
    }

    pub async fn get_watches(&self) -> Result<Vec<sqlite::WatchCondition>> {
        self.persistent.get_watch_conditions()
    }

    pub fn db_size_kb(&self) -> u64 { self.persistent.db_size_kb() }
}
