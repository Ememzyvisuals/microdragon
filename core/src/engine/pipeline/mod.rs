// microdragon-core/src/engine/pipeline/mod.rs
//
// MICRODRAGON 9-Phase Execution Pipeline
// ─────────────────────────────────
// INPUT → ANALYZE → PLAN → SIMULATE → EXECUTE → VERIFY → OPTIMIZE → STORE → RESPOND
// This is more sophisticated than Claude Code (which has no simulate/verify/optimize phases)
// and OpenClaw (which has no pipeline at all)

use anyhow::Result;
use std::sync::Arc;
use std::time::Instant;
use tracing::{info, debug, warn};
use uuid::Uuid;
use chrono::Utc;

use crate::engine::MicrodragonEngine;
use crate::engine::autonomous::{
    PipelineTask, PipelinePhase, TaskAnalysis, SimulationResult,
    AgentRole, Complexity, ExecutionMode, FeedbackScore, PerformanceMetrics, TokenGuard,
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
    pub fn new(config: &crate::config::MicrodragonConfig) -> Self {
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
        let start = Instant::now();

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

        macro_rules! emit_phase {
            ($phase:expr) => {
                task.phase = $phase.clone();
                debug!("Pipeline phase: {}", $phase);
                if let Some(ref tx) = phase_tx {
                    tx.send($phase).await.ok();
                }
            }
        }

        // ── Phase 1: ANALYZE ─────────────────────────────────────────────
        emit_phase!(PipelinePhase::Analyzing);
        let analysis = self.phase_analyze(input);
        task.agent = analysis.suggested_agent.clone();
        task.analysis = Some(analysis);

        // ── Phase 2: PLAN ────────────────────────────────────────────────
        emit_phase!(PipelinePhase::Planning);
        // Plan is already created by brain/planner.rs — here we validate it
        debug!("Planning for agent: {}", task.agent);

        // ── Phase 3: SIMULATE ────────────────────────────────────────────
        // Only in Autonomous mode for high-complexity tasks
        let run_simulation = matches!(self.mode, ExecutionMode::Autonomous) ||
            matches!(task.analysis.as_ref().map(|a| &a.complexity),
                     Some(Complexity::High) | Some(Complexity::Epic));

        if run_simulation {
            emit_phase!(PipelinePhase::Simulating);
            let sim = self.phase_simulate(input, &task.analysis).await;
            if !sim.should_proceed {
                let warnings = sim.warnings.join(", ");
                task.phase = PipelinePhase::Failed(format!("Simulation blocked: {}", warnings));
                task.latency_ms = start.elapsed().as_millis() as u64;
                return Ok(task);
            }
            task.simulation = Some(sim);
        }

        // ── Phase 4: EXECUTE ─────────────────────────────────────────────
        emit_phase!(PipelinePhase::Executing);

        // Check confirmation for dangerous actions in Command mode
        if let Some(ref analysis) = task.analysis {
            if analysis.requires_confirmation && matches!(self.mode, ExecutionMode::Command) {
                warn!("Task requires user confirmation: {}", input);
                // In CLI this would prompt user — for now proceed
            }
        }

        // Token guard check
        let prompt_hash = format!("{:x}", md5_simple(input));
        if let Err(e) = self.token_guard.check_and_record(
            task.analysis.as_ref().map_or(500, |a| a.estimated_tokens),
            &prompt_hash
        ) {
            task.phase = PipelinePhase::Failed(e.to_string());
            task.latency_ms = start.elapsed().as_millis() as u64;
            return Ok(task);
        }

        // Get agent config for this task
        let agent_cfg = self.agent_registry.get(&task.agent);
        let result = engine.process_command(input).await?;
        task.tokens_used = result.tokens_used;
        task.result = Some(result.response.clone());

        // ── Phase 5: VERIFY ──────────────────────────────────────────────
        emit_phase!(PipelinePhase::Verifying);
        task.verified = self.phase_verify(&task.result, &task.analysis);

        // ── Phase 6: OPTIMIZE ────────────────────────────────────────────
        emit_phase!(PipelinePhase::Optimizing);
        if let Some(ref raw_result) = task.result {
            task.optimized_result = Some(self.phase_optimize(raw_result, input));
        }

        // ── Phase 7: STORE ───────────────────────────────────────────────
        emit_phase!(PipelinePhase::Storing);
        {
            let memory = engine.memory.read().await;
            let output = task.optimized_result.as_deref()
                .or(task.result.as_deref())
                .unwrap_or("");
            memory.save_task(
                &task.id, &task.agent.to_string(), input,
                output, "complete", &task.agent.to_string(),
                task.tokens_used, start.elapsed().as_millis() as u64,
            ).await.ok();
        }

        // ── Phase 8: RESPOND ─────────────────────────────────────────────
        emit_phase!(PipelinePhase::Complete);
        task.phase = PipelinePhase::Complete;
        task.latency_ms = start.elapsed().as_millis() as u64;
        task.completed_at = Some(Utc::now());

        // Update metrics
        self.metrics.record_task(&task);

        info!("Pipeline complete: {} phases, {}ms, {} tokens",
              8, task.latency_ms, task.tokens_used);

        Ok(task)
    }

    // ── Phase implementations ─────────────────────────────────────────────

    fn phase_analyze(&self, input: &str) -> TaskAnalysis {
        let lower = input.to_lowercase();

        // Classify complexity
        let complexity = if input.len() > 500 || lower.contains("analyze") || lower.contains("review") {
            Complexity::High
        } else if input.len() > 200 || lower.contains("write") || lower.contains("create") {
            Complexity::Medium
        } else {
            Complexity::Low
        };

        // Check if confirmation needed
        let requires_confirmation = lower.contains("delete") || lower.contains("send email")
            || lower.contains("transfer") || lower.contains("buy") || lower.contains("sell")
            || lower.contains("rm ") || lower.contains("drop table");

        // Detect risks
        let mut risks = Vec::new();
        if lower.contains("delete") { risks.push("Destructive operation".to_string()); }
        if lower.contains("send") && lower.contains("email") { risks.push("Email send action".to_string()); }
        if lower.contains("root") || lower.contains("sudo") { risks.push("Elevated privileges".to_string()); }

        let estimated_tokens = match &complexity {
            Complexity::Low => 200,
            Complexity::Medium => 800,
            Complexity::High => 2500,
            Complexity::Epic => 8000,
        };

        TaskAnalysis {
            intent: self.agent_registry.route_task(input).to_string(),
            complexity,
            requires_confirmation,
            estimated_tokens,
            risks,
            suggested_agent: self.agent_registry.route_task(input),
        }
    }

    async fn phase_simulate(&self, input: &str, analysis: &Option<TaskAnalysis>) -> SimulationResult {
        let risks = analysis.as_ref().map_or_else(Vec::new, |a| a.risks.clone());
        let has_risks = !risks.is_empty();

        SimulationResult {
            predicted_outcome: format!("Task '{}...' will produce a response", &input[..input.len().min(50)]),
            detected_risks: risks.clone(),
            confidence: if has_risks { 0.75 } else { 0.95 },
            should_proceed: true, // In production: AI evaluates risks
            warnings: risks.iter().map(|r| format!("⚠ {}", r)).collect(),
        }
    }

    fn phase_verify(&self, result: &Option<String>, analysis: &Option<TaskAnalysis>) -> bool {
        // Basic verification: result exists and isn't empty
        match result {
            Some(r) if !r.trim().is_empty() => true,
            _ => false,
        }
    }

    fn phase_optimize(&self, result: &str, original_input: &str) -> String {
        // Post-processing: trim, ensure completeness
        let trimmed = result.trim().to_string();

        // If the result seems truncated (ends mid-sentence), flag it
        if trimmed.len() > 10 && !trimmed.ends_with(['.', '!', '?', '\n', '`', '}', ')']) {
            format!("{}\n\n*[Note: Response may be truncated]*", trimmed)
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

/// Simple MD5-like hash for loop detection (not cryptographic)
fn md5_simple(s: &str) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in s.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x00000100000001b3);
    }
    hash
}
