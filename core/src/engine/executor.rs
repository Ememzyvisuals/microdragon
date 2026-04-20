// microdragon-core/src/engine/executor.rs
use anyhow::Result;
use tracing::info;
use super::task::{Task, TaskResult, TaskStatus};

pub struct TaskExecutor;

impl TaskExecutor {
    pub fn new() -> Self { Self }

    pub async fn execute(&self, mut task: Task) -> Result<Task> {
        info!("Executing task {}", task.id);
        task.status = TaskStatus::Running;

        // Task execution is handled by brain + dispatchers
        task.status = TaskStatus::Completed;
        Ok(task)
    }
}
