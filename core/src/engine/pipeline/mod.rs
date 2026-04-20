// microdragon-core/src/engine/pipeline/mod.rs
// 9-Phase Execution Pipeline

use anyhow::Result;
use std::sync::Arc;
use std::time::Instant;
use tracing::{info, debug, warn};
use uuid::Uuid;
use chrono::Utc;

use crate::engine::MicrodragonEngine;
use crate::engine::autonomous::{
    PipelineTask, PipelinePhase, TaskAnalysis, SimulationResult,
    AgentRole, Complexity, ExecutionMode, PerformanceMetrics, TokenGuard,
};
use crate::engine::agents::AgentRegistry;

pub struct Pipeline {
    pub id: String,
    mode: ExecutionMode,
    token_guard: TokenGuard,
    agent_registry: AgentRegistry,
    metrics: PerformanceMetrics,
}

impl Pipeline {
    pub fn new(_config: &crate::config::MicrodragonConfig) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            mode: ExecutionMode::Command,
            token_guard: TokenGuard::new(),
            agent_registry: AgentRegistry::new(),
            metrics: PerformanceMetrics::default(),
        }
    }

    pub fn set_mode(&mut self, mode: ExecutionMode) {
        info!("Pipeline mode: {:?}", mode);
        self.mode = mode;
    }

    /// Execute the full 9-phase pipeline
    pub async fn run(
        &mut self,
        input: &str,
        engine: &Arc<MicrodragonEngine>,
        phase_tx: Option<tokio::sync::mpsc::Sender<PipelinePhase>>,
    ) -> Result<PipelineTask> {
        let task_id = Uuid::new_v4().to_string();
        let start   = Instant::now();

        let mut task = PipelineTask {
            id: task_id.clone(),
            input: input.to_string(),
            mode: self.mode.clone(),
            phase: PipelinePhase::Analyzing,
            analysis: None,
            simulation: None,
            result: None,
            verified: false,
            optimized_result: None,
            agent: AgentRole::Master,
            tokens_used: 0,
            latency_ms: 0,
            created_at: Utc::now(),
            completed_at: None,
            user_feedback: None,
        };

        // Helper macro — sends phase update to TUI/progress bar
        macro_rules! emit {
            ($p:expr) => {
                task.phase = $p.clone();
                debug!("Phase: {}", $p);
                if let Some(ref tx) = phase_tx {
                    tx.send($p).await.ok();
                }
            };
        }

        // ── Phase 1: ANALYZE ─────────────────────────────────────────────────
        emit!(PipelinePhase::Analyzing);
        let analysis = self.phase_analyze(input);
        task.agent   = analysis.suggested_agent.clone();
        task.analysis = Some(analysis);

        // ── Phase 2: PLAN ────────────────────────────────────────────────────
        emit!(PipelinePhase::Planning);
        debug!("Agent: {}", task.agent);

        // ── Phase 3: SIMULATE (Autonomous + High complexity only) ────────────
        let needs_sim = matches!(self.mode, ExecutionMode::Autonomous)
            || matches!(
                task.analysis.as_ref().map(|a| &a.complexity),
                Some(Complexity::High) | Some(Complexity::Epic)
            );

        if needs_sim {
            emit!(PipelinePhase::Simulating);
            let sim = self.phase_simulate(input, &task.analysis).await;
            if !sim.should_proceed {
                let msg = sim.warnings.join(", ");
                task.phase    = PipelinePhase::Failed(format!("Blocked: {}", msg));
                task.latency_ms = start.elapsed().as_millis() as u64;
                return Ok(task);
            }
            task.simulation = Some(sim);
        }

        // ── Phase 4: EXECUTE ─────────────────────────────────────────────────
        emit!(PipelinePhase::Executing);

        if let Some(ref analysis) = task.analysis {
            if analysis.requires_confirmation && matches!(self.mode, ExecutionMode::Command) {
                warn!("Confirmation required for: {}", input);
            }
        }

        let hash = hash_str(input);
        if let Err(e) = self.token_guard.check_and_record(
            task.analysis.as_ref().map_or(500, |a| a.estimated_tokens),
            &hash,
        ) {
            task.phase    = PipelinePhase::Failed(e.to_string());
            task.latency_ms = start.elapsed().as_millis() as u64;
            return Ok(task);
        }

        let _agent_cfg = self.agent_registry.get(&task.agent);
        let result     = engine.process_command(input).await?;
        task.tokens_used = result.tokens_used;
        task.result    = Some(result.response.clone());

        // ── Phase 5: VERIFY ──────────────────────────────────────────────────
        emit!(PipelinePhase::Verifying);
        task.verified = self.phase_verify(&task.result);

        // ── Phase 6: OPTIMIZE ────────────────────────────────────────────────
        emit!(PipelinePhase::Optimizing);
        if let Some(ref raw) = task.result {
            task.optimized_result = Some(self.phase_optimize(raw));
        }

        // ── Phase 7: STORE ───────────────────────────────────────────────────
        emit!(PipelinePhase::Storing);
        {
            let memory = engine.memory.read().await;
            let out    = task.optimized_result.as_deref()
                .or(task.result.as_deref()).unwrap_or("");
            memory.save_task(
                &task.id, &task.agent.to_string(), input, out,
                "complete", &task.agent.to_string(),
                task.tokens_used, start.elapsed().as_millis() as u64,
            ).await.ok();
        }

        // ── Phase 8: RESPOND ─────────────────────────────────────────────────
        emit!(PipelinePhase::Complete);
        task.phase      = PipelinePhase::Complete;
        task.latency_ms = start.elapsed().as_millis() as u64;
        task.completed_at = Some(Utc::now());
        self.metrics.record_task(&task);

        info!("Pipeline done: {}ms, {} tokens", task.latency_ms, task.tokens_used);
        Ok(task)
    }

    // ── Phase helpers ─────────────────────────────────────────────────────────

    fn phase_analyze(&self, input: &str) -> TaskAnalysis {
        let lower = input.to_lowercase();

        let complexity = if input.len() > 500
            || lower.contains("analyze") || lower.contains("review")
        {
            Complexity::High
        } else if input.len() > 200
            || lower.contains("write") || lower.contains("create")
        {
            Complexity::Medium
        } else {
            Complexity::Low
        };

        let requires_confirmation =
            lower.contains("delete") || lower.contains("send email")
            || lower.contains("transfer") || lower.contains("buy")
            || lower.contains("sell") || lower.contains("rm ")
            || lower.contains("drop table");

        let mut risks = Vec::new();
        if lower.contains("delete") {
            risks.push("Destructive operation".to_string());
        }
        if lower.contains("send") && lower.contains("email") {
            risks.push("Email send action".to_string());
        }
        if lower.contains("sudo") {
            risks.push("Elevated privileges".to_string());
        }

        let estimated_tokens = match complexity {
            Complexity::Low    =>   200,
            Complexity::Medium =>   800,
            Complexity::High   =>  2500,
            Complexity::Epic   =>  8000,
        };

        TaskAnalysis {
            intent:                 self.agent_registry.route_task(input).to_string(),
            complexity,
            requires_confirmation,
            estimated_tokens,
            risks,
            suggested_agent:        self.agent_registry.route_task(input),
        }
    }

    async fn phase_simulate(
        &self,
        input: &str,
        analysis: &Option<TaskAnalysis>,
    ) -> SimulationResult {
        let risks     = analysis.as_ref().map_or_else(Vec::new, |a| a.risks.clone());
        let has_risks = !risks.is_empty();
        SimulationResult {
            predicted_outcome: format!(
                "Task '{:.50}' will produce a response",
                input
            ),
            detected_risks: risks.clone(),
            confidence:     if has_risks { 0.75 } else { 0.95 },
            should_proceed: true,
            warnings:       risks.iter().map(|r| format!("  {}", r)).collect(),
        }
    }

    fn phase_verify(&self, result: &Option<String>) -> bool {
        matches!(result, Some(r) if !r.trim().is_empty())
    }

    fn phase_optimize(&self, result: &str) -> String {
        let trimmed = result.trim().to_string();
        // Flag potentially truncated responses
        let last = trimmed.chars().last();
        let looks_complete = matches!(last, Some('.' | '!' | '?' | '\n' | '`' | '}' | ')'));
        if trimmed.len() > 10 && !looks_complete {
            format!("{}\n\n*[Note: response may be truncated]*", trimmed)
        } else {
            trimmed
        }
    }

    pub fn get_metrics(&self) -> &PerformanceMetrics {
        &self.metrics
    }

    pub fn current_mode(&self) -> &ExecutionMode {
        &self.mode
    }
}

/// FNV-1a hash for loop detection (fast, non-cryptographic)
fn hash_str(s: &str) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in s.bytes() {
        h ^= b as u64;
        h  = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{:x}", h)
}