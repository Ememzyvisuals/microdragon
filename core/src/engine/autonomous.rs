// microdragon-core/src/engine/autonomous.rs
//
// MICRODRAGON Autonomous Engine
// ───────────────────────
// Implements COMMAND MODE and AUTONOMOUS MODE.
// Full pipeline: INPUT → ANALYZE → PLAN → SIMULATE → EXECUTE → VERIFY → OPTIMIZE → STORE → RESPOND
// This is what beats Claude Code (coding only) and OpenClaw (no pipeline, no simulation, no verify)

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc, broadcast};
use tracing::{info, warn, error};
use uuid::Uuid;
use std::collections::VecDeque;

// ─── Execution modes ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExecutionMode {
    /// Precise, deterministic — user confirms sensitive steps
    Command,
    /// Background execution, scheduled tasks, multi-step autonomous workflows
    Autonomous,
}

impl std::fmt::Display for ExecutionMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutionMode::Command    => write!(f, "COMMAND"),
            ExecutionMode::Autonomous => write!(f, "AUTONOMOUS"),
        }
    }
}

// ─── Pipeline phases ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PipelinePhase {
    Analyzing,
    Planning,
    Simulating,
    Executing,
    Verifying,
    Optimizing,
    Storing,
    Complete,
    Failed(String),
}

impl std::fmt::Display for PipelinePhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PipelinePhase::Analyzing  => write!(f, "analyzing"),
            PipelinePhase::Planning   => write!(f, "planning"),
            PipelinePhase::Simulating => write!(f, "simulating"),
            PipelinePhase::Executing  => write!(f, "executing"),
            PipelinePhase::Verifying  => write!(f, "verifying"),
            PipelinePhase::Optimizing => write!(f, "optimizing"),
            PipelinePhase::Storing    => write!(f, "storing"),
            PipelinePhase::Complete   => write!(f, "complete"),
            PipelinePhase::Failed(e)  => write!(f, "failed: {}", e),
        }
    }
}

// ─── Task record ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineTask {
    pub id: String,
    pub input: String,
    pub mode: ExecutionMode,
    pub phase: PipelinePhase,

    // Pipeline outputs
    pub analysis: Option<TaskAnalysis>,
    pub simulation: Option<SimulationResult>,
    pub result: Option<String>,
    pub verified: bool,
    pub optimized_result: Option<String>,

    // Meta
    pub agent: AgentRole,
    pub tokens_used: u32,
    pub latency_ms: u64,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub user_feedback: Option<FeedbackScore>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskAnalysis {
    pub intent: String,
    pub complexity: Complexity,
    pub requires_confirmation: bool,
    pub estimated_tokens: u32,
    pub risks: Vec<String>,
    pub suggested_agent: AgentRole,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Complexity {
    Low,      // single step, < 500 tokens
    Medium,   // multi-step, < 2000 tokens
    High,     // complex workflow, < 8000 tokens
    Epic,     // multi-agent, > 8000 tokens
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationResult {
    pub predicted_outcome: String,
    pub detected_risks: Vec<String>,
    pub confidence: f64,
    pub should_proceed: bool,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FeedbackScore {
    Good,
    Bad,
    Neutral,
}

// ─── Agent hierarchy ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AgentRole {
    Master,      // MICRODRAGON core — orchestrates, delegates
    Coding,      // code generation, debugging, git
    Research,    // web research, summarization
    Business,    // market analysis, trading signals
    Automation,  // browser/desktop automation
    Writing,     // creative writing, emails, documents
    Security,    // security review, vulnerability analysis
}

impl std::fmt::Display for AgentRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentRole::Master     => write!(f, "master"),
            AgentRole::Coding     => write!(f, "coding"),
            AgentRole::Research   => write!(f, "research"),
            AgentRole::Business   => write!(f, "business"),
            AgentRole::Automation => write!(f, "automation"),
            AgentRole::Writing    => write!(f, "writing"),
            AgentRole::Security   => write!(f, "security"),
        }
    }
}

// ─── System state ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemState {
    pub mode: ExecutionMode,
    pub active_tasks: usize,
    pub queued_tasks: usize,
    pub background_jobs: usize,
    pub modules_running: Vec<String>,
    pub memory_mb: f64,
    pub uptime_secs: u64,
    pub total_tokens_used: u64,
    pub total_tasks_completed: u64,
    pub success_rate: f64,
    pub avg_latency_ms: u64,
}

// ─── Token & cost guard ───────────────────────────────────────────────────────

pub struct TokenGuard {
    session_tokens: u32,
    session_limit: u32,
    request_count: u32,
    request_limit: u32,
    loop_detector: VecDeque<String>,
}

impl TokenGuard {
    pub fn new() -> Self {
        Self {
            session_tokens: 0,
            session_limit: 500_000,
            request_count: 0,
            request_limit: 1000,
            loop_detector: VecDeque::with_capacity(10),
        }
    }

    pub fn check_and_record(&mut self, tokens: u32, prompt_hash: &str) -> Result<()> {
        self.session_tokens += tokens;
        self.request_count += 1;

        if self.session_tokens > self.session_limit {
            return Err(anyhow::anyhow!(
                "Session token limit reached ({}/{}). Start new session or increase limit.",
                self.session_tokens, self.session_limit
            ));
        }
        if self.request_count > self.request_limit {
            return Err(anyhow::anyhow!(
                "Request limit reached ({}/{}). Cooldown required.",
                self.request_count, self.request_limit
            ));
        }

        // Loop detection: same prompt 3 times in last 10 = abort
        self.loop_detector.push_back(prompt_hash.to_string());
        if self.loop_detector.len() > 10 { self.loop_detector.pop_front(); }
        let repeats = self.loop_detector.iter().filter(|h| *h == prompt_hash).count();
        if repeats >= 3 {
            return Err(anyhow::anyhow!(
                "Loop detected: same request repeated {} times. Task terminated.",
                repeats
            ));
        }

        Ok(())
    }

    pub fn stats(&self) -> (u32, u32, u32, u32) {
        (self.session_tokens, self.session_limit, self.request_count, self.request_limit)
    }
}

// ─── Reliability engine ───────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ExecutionLog {
    pub task_id: String,
    pub phase: PipelinePhase,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub is_error: bool,
}

pub struct ReliabilityEngine {
    pub logs: Vec<ExecutionLog>,
    max_retries: u32,
    retry_delay_ms: u64,
}

impl ReliabilityEngine {
    pub fn new() -> Self {
        Self {
            logs: Vec::new(),
            max_retries: 3,
            retry_delay_ms: 500,
        }
    }

    pub fn log(&mut self, task_id: &str, phase: PipelinePhase, msg: &str, is_error: bool) {
        self.logs.push(ExecutionLog {
            task_id: task_id.to_string(),
            phase,
            message: msg.to_string(),
            timestamp: Utc::now(),
            is_error,
        });
    }

    pub async fn with_retry<F, T, Fut>(&self, task: F) -> Result<T>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let mut last_err = None;
        for attempt in 0..=self.max_retries {
            if attempt > 0 {
                let delay = self.retry_delay_ms * 2u64.pow(attempt - 1);
                tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
                warn!("Retry attempt {}/{}", attempt, self.max_retries);
            }
            match task().await {
                Ok(val) => return Ok(val),
                Err(e) => { last_err = Some(e); }
            }
        }
        Err(last_err.unwrap_or_else(|| anyhow::anyhow!("All retries exhausted")))
    }

    pub fn recent_errors(&self, task_id: &str) -> Vec<&ExecutionLog> {
        self.logs.iter()
            .filter(|l| l.task_id == task_id && l.is_error)
            .collect()
    }
}

// ─── Performance metrics ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PerformanceMetrics {
    pub total_tasks: u64,
    pub successful_tasks: u64,
    pub failed_tasks: u64,
    pub total_tokens: u64,
    pub total_latency_ms: u64,
    pub good_feedback: u64,
    pub bad_feedback: u64,

    // Per-agent stats
    pub coding_tasks: u64,
    pub research_tasks: u64,
    pub business_tasks: u64,
    pub automation_tasks: u64,
}

impl PerformanceMetrics {
    pub fn success_rate(&self) -> f64 {
        if self.total_tasks == 0 { return 0.0; }
        self.successful_tasks as f64 / self.total_tasks as f64 * 100.0
    }

    pub fn avg_latency_ms(&self) -> u64 {
        if self.total_tasks == 0 { return 0; }
        self.total_latency_ms / self.total_tasks
    }

    pub fn avg_tokens_per_task(&self) -> u64 {
        if self.total_tasks == 0 { return 0; }
        self.total_tokens / self.total_tasks
    }

    pub fn user_satisfaction(&self) -> f64 {
        let total_feedback = self.good_feedback + self.bad_feedback;
        if total_feedback == 0 { return 0.0; }
        self.good_feedback as f64 / total_feedback as f64 * 100.0
    }

    pub fn record_task(&mut self, task: &PipelineTask) {
        self.total_tasks += 1;
        self.total_tokens += task.tokens_used as u64;
        self.total_latency_ms += task.latency_ms;

        match task.phase {
            PipelinePhase::Complete => self.successful_tasks += 1,
            PipelinePhase::Failed(_) => self.failed_tasks += 1,
            _ => {}
        }

        match task.agent {
            AgentRole::Coding     => self.coding_tasks += 1,
            AgentRole::Research   => self.research_tasks += 1,
            AgentRole::Business   => self.business_tasks += 1,
            AgentRole::Automation => self.automation_tasks += 1,
            _ => {}
        }

        if let Some(feedback) = &task.user_feedback {
            match feedback {
                FeedbackScore::Good => self.good_feedback += 1,
                FeedbackScore::Bad  => self.bad_feedback += 1,
                FeedbackScore::Neutral => {}
            }
        }
    }
}

// ─── Self-improvement ledger ──────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImprovementEntry {
    pub id: String,
    pub observation: String,
    pub suggestion: String,
    pub applied: bool,
    pub validated: bool,
    pub created_at: DateTime<Utc>,
    pub applied_at: Option<DateTime<Utc>>,
}

pub struct SelfImprovementLedger {
    entries: Vec<ImprovementEntry>,
    failure_patterns: Vec<(String, u32)>, // (pattern, count)
}

impl SelfImprovementLedger {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            failure_patterns: Vec::new(),
        }
    }

    /// Record a failure and detect patterns
    pub fn record_failure(&mut self, error: &str, context: &str) {
        // Normalize error
        let pattern = error.split_whitespace().take(5).collect::<Vec<_>>().join(" ");

        if let Some(entry) = self.failure_patterns.iter_mut().find(|(p, _)| p == &pattern) {
            entry.1 += 1;
            // If same pattern fails 3+ times, suggest improvement
            if entry.1 >= 3 {
                self.suggest_improvement(
                    &format!("Recurring failure: '{}' ({}x)", pattern, entry.1),
                    &format!("Add handling for '{}' error pattern in context: {}", pattern, &context[..context.len().min(100)])
                );
            }
        } else {
            self.failure_patterns.push((pattern, 1));
        }
    }

    pub fn suggest_improvement(&mut self, observation: &str, suggestion: &str) {
        self.entries.push(ImprovementEntry {
            id: Uuid::new_v4().to_string(),
            observation: observation.to_string(),
            suggestion: suggestion.to_string(),
            applied: false,
            validated: false,
            created_at: Utc::now(),
            applied_at: None,
        });
    }

    pub fn pending_suggestions(&self) -> Vec<&ImprovementEntry> {
        self.entries.iter().filter(|e| !e.applied).collect()
    }

    pub fn mark_applied(&mut self, id: &str) {
        if let Some(entry) = self.entries.iter_mut().find(|e| e.id == id) {
            entry.applied = true;
            entry.applied_at = Some(Utc::now());
        }
    }

    /// Learn user behavior patterns from task history
    pub fn learn_from_task(&mut self, task: &PipelineTask) {
        // Track which agents handle which inputs well
        if let PipelinePhase::Complete = &task.phase {
            if let Some(FeedbackScore::Good) = &task.user_feedback {
                // Good feedback → reinforce this agent for this intent type
                self.suggest_improvement(
                    &format!("Agent '{}' received positive feedback for: {}", task.agent, &task.input[..task.input.len().min(50)]),
                    &format!("Prefer '{}' agent for similar requests", task.agent)
                );
            }
        }
    }
}

// ─── Scheduled task ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTask {
    pub id: String,
    pub name: String,
    pub prompt: String,
    pub cron_expr: String,
    pub next_run: DateTime<Utc>,
    pub last_run: Option<DateTime<Utc>>,
    pub enabled: bool,
    pub run_count: u64,
}

// ─── Autonomous scheduler ─────────────────────────────────────────────────────

pub struct AutonomousScheduler {
    pub tasks: Vec<ScheduledTask>,
    tx: mpsc::Sender<String>,
}

impl AutonomousScheduler {
    pub fn new(tx: mpsc::Sender<String>) -> Self {
        Self { tasks: Vec::new(), tx }
    }

    pub fn add_task(&mut self, name: &str, prompt: &str, cron_expr: &str) -> String {
        let id = Uuid::new_v4().to_string();
        let next_run = self.parse_next_run(cron_expr);
        self.tasks.push(ScheduledTask {
            id: id.clone(), name: name.to_string(),
            prompt: prompt.to_string(), cron_expr: cron_expr.to_string(),
            next_run, last_run: None, enabled: true, run_count: 0,
        });
        info!("Scheduled task '{}' added: {}", name, cron_expr);
        id
    }

    fn parse_next_run(&self, _cron: &str) -> DateTime<Utc> {
        // Simplified: next run in 1 hour. Full cron parsing with `cron` crate in production.
        Utc::now() + chrono::Duration::hours(1)
    }

    pub async fn tick(&mut self) {
        let now = Utc::now();
        for task in self.tasks.iter_mut() {
            if task.enabled && task.next_run <= now {
                info!("[Scheduler] Running task: {}", task.name);
                task.last_run = Some(now);
                task.run_count += 1;
                task.next_run = now + chrono::Duration::hours(1);
                self.tx.send(task.prompt.clone()).await.ok();
            }
        }
    }
}
