// microdragon-core/src/engine/pipeline.rs
//
// ╔══════════════════════════════════════════════════════════════════════╗
// ║       MICRODRAGON  —  9-PHASE AGENTIC PIPELINE                      ║
// ║                                                                      ║
// ║  Phase 1 · PERCEIVE   — Intent parsing & task classification         ║
// ║  Phase 2 · PLAN       — Build step-by-step execution strategy        ║
// ║  Phase 3 · GATHER     — Run real tools (web, files, memory)          ║
// ║  Phase 4 · SYNTHESIZE — Merge all data into one rich context         ║
// ║  Phase 5 · REASON     — Primary AI generation with full context      ║
// ║  Phase 6 · REFLECT    — Self-critique: completeness & accuracy       ║
// ║  Phase 7 · REFINE     — Re-generate if reflection flags issues       ║
// ║  Phase 8 · FORMAT     — Structure output for agentic presentation    ║
// ║  Phase 9 · COMMIT     — Persist to memory, emit telemetry            ║
// ╚══════════════════════════════════════════════════════════════════════╝
//
// © 2026 EMEMZYVISUALS DIGITALS — Emmanuel Ariyo

use anyhow::Result;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::{info, debug};

use crate::brain::MicrodragonBrain;
use crate::brain::intent::{IntentParser, IntentType, ParsedIntent};
use crate::config::providers::{ChatMessage, CompletionRequest, MessageRole};
use crate::memory::MemoryStore;
use crate::tools::web_search::{WebSearcher, format_for_ai as fmt_search};
use crate::tools::file_reader;
use crate::tools::shell;
use crate::events::{AgentEvent, EventKind, EventTx};

// ─── Pipeline Output ─────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct PipelineResult {
    pub response:      String,
    pub model:         String,
    pub provider:      String,
    pub tokens_used:   u32,
    pub latency_ms:    u64,
    pub phases:        Vec<PhaseRecord>,
    pub tools_used:    Vec<String>,
    pub web_results:   usize,
    pub commands_run:  Vec<shell::ShellResult>,
}

#[derive(Debug, Clone)]
pub struct PhaseRecord {
    pub phase:       u8,
    pub name:        &'static str,
    pub status:      PhaseStatus,
    pub detail:      String,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PhaseStatus {
    Ok,
    Skipped,
    Warning,
}

// ─── Pipeline ────────────────────────────────────────────────────────────────

pub struct AgenticPipeline {
    brain:         Arc<MicrodragonBrain>,
    memory:        Arc<RwLock<MemoryStore>>,
    searcher:      WebSearcher,
    intent_parser: IntentParser,
    /// Optional TUI event channel — if set, pipeline emits live events to TUI
    event_tx:      Option<EventTx>,
}

impl AgenticPipeline {
    pub fn new(brain: Arc<MicrodragonBrain>, memory: Arc<RwLock<MemoryStore>>) -> Self {
        Self {
            brain,
            memory,
            searcher: WebSearcher::new(),
            intent_parser: IntentParser::new(),
            event_tx: None,
        }
    }

    pub fn with_event_tx(mut self, tx: EventTx) -> Self {
        self.event_tx = Some(tx);
        self
    }

    /// Emit an event to the TUI if connected
    fn emit(&self, kind: EventKind, text: impl Into<String>) {
        if let Some(tx) = &self.event_tx {
            let _ = tx.send(AgentEvent::new(kind, text));
        }
    }

    pub async fn run(
        &self,
        input: &str,
        context: &[ChatMessage],
        progress: &(dyn Fn(u8, &'static str, &str) + Send + Sync),
    ) -> Result<PipelineResult> {

        let total_start = Instant::now();
        let mut phases: Vec<PhaseRecord> = Vec::new();
        let mut tools_used: Vec<String> = Vec::new();

        // ────────────────────────────────────────────────────────────────────
        // PHASE 1 — PERCEIVE
        // ────────────────────────────────────────────────────────────────────
        progress(1, "PERCEIVE", "Classifying intent and extracting entities…");
        let p1 = Instant::now();

        let intent = self.intent_parser.parse(input).await?;
        let task_type_label = intent.intent_type.to_string().to_uppercase();
        let phase1_detail = format!(
            "intent={} entities={} tools_needed={}",
            task_type_label,
            intent.entities.len(),
            intent.requires_tools.join(",")
        );

        phases.push(PhaseRecord {
            phase: 1, name: "PERCEIVE",
            status: PhaseStatus::Ok,
            detail: phase1_detail,
            duration_ms: p1.elapsed().as_millis() as u64,
        });

        // ────────────────────────────────────────────────────────────────────
        // PHASE 2 — PLAN
        // ────────────────────────────────────────────────────────────────────
        progress(2, "PLAN", &format!("Building execution strategy for {} task…", task_type_label));
        let p2 = Instant::now();

        let plan = self.build_plan(&intent, input);
        let plan_display = plan.iter()
            .enumerate()
            .map(|(i, s)| format!("{}. {}", i + 1, s))
            .collect::<Vec<_>>()
            .join(" → ");

        phases.push(PhaseRecord {
            phase: 2, name: "PLAN",
            status: PhaseStatus::Ok,
            detail: format!("{} steps: {}", plan.len(), plan_display),
            duration_ms: p2.elapsed().as_millis() as u64,
        });

        // ────────────────────────────────────────────────────────────────────
        // PHASE 3 — GATHER
        // ────────────────────────────────────────────────────────────────────
        progress(3, "GATHER", "Running tools — web search, memory recall, file analysis…");
        let p3 = Instant::now();

        let mut gathered_context = String::new();
        let mut web_result_count = 0usize;

        // 3a. Memory recall
        let memory_snippets = {
            let mem = self.memory.read().await;
            mem.search_relevant(input, 3).await.unwrap_or_default()
        };
        if !memory_snippets.is_empty() {
            tools_used.push("memory_recall".to_string());
            gathered_context.push_str("## Relevant Memory\n");
            for snippet in &memory_snippets {
                gathered_context.push_str(&format!("- {}\n", snippet));
            }
            gathered_context.push('\n');
        }

        // 3b. Web search — for research, business, and any question about current events
        let needs_web = matches!(
            intent.intent_type,
            IntentType::Research | IntentType::Business
        ) || input_needs_web(input);

        if needs_web {
            let queries = self.build_search_queries(&intent, input);
            progress(3, "GATHER", &format!("Searching web: \"{}\"…", queries[0]));

            for query in &queries {
                match self.searcher.search(query, 5).await {
                    Ok(results) if !results.is_empty() => {
                        web_result_count += results.len();
                        gathered_context.push_str(&fmt_search(query, &results));
                        gathered_context.push('\n');
                        if !tools_used.contains(&"web_search".to_string()) {
                            tools_used.push("web_search".to_string());
                        }
                    }
                    Ok(_) => {
                        debug!("No web results for query: {}", query);
                    }
                    Err(e) => {
                        debug!("Web search failed: {}", e);
                    }
                }
            }
        }

        // 3c. File analysis — if input references a file path
        if let Some(file_path) = extract_file_path(input) {
            progress(3, "GATHER", &format!("Reading file: {}…", file_path));
            match file_reader::read_file(&file_path) {
                Ok(fc) => {
                    tools_used.push("file_reader".to_string());
                    gathered_context.push_str(&format!(
                        "## File Contents: {} ({:.1}KB, {} lines)\n```{}\n{}\n```\n\n",
                        fc.path, fc.size_kb, fc.line_count, fc.extension, fc.content
                    ));
                }
                Err(e) => {
                    gathered_context.push_str(&format!("## File Error\n{}\n\n", e));
                }
            }
        }

        phases.push(PhaseRecord {
            phase: 3, name: "GATHER",
            status: PhaseStatus::Ok,
            detail: format!(
                "tools=[{}] web_results={} memory_snippets={}",
                tools_used.join(","),
                web_result_count,
                memory_snippets.len()
            ),
            duration_ms: p3.elapsed().as_millis() as u64,
        });

        // ────────────────────────────────────────────────────────────────────
        // PHASE 4 — SYNTHESIZE
        // ────────────────────────────────────────────────────────────────────
        progress(4, "SYNTHESIZE", "Merging all data into execution context…");
        let p4 = Instant::now();

        let system_prompt = self.build_system_prompt(&intent, &tools_used);
        let enriched_user_message = if gathered_context.is_empty() {
            input.to_string()
        } else {
            format!(
                "{}\n\n---\n## Gathered Intelligence\n\n{}\n---\n\nNow respond to: {}",
                self.task_preamble(&intent),
                gathered_context,
                input
            )
        };

        phases.push(PhaseRecord {
            phase: 4, name: "SYNTHESIZE",
            status: PhaseStatus::Ok,
            detail: format!(
                "context_tokens~{} gathered_bytes={}",
                (system_prompt.len() + enriched_user_message.len()) / 4,
                gathered_context.len()
            ),
            duration_ms: p4.elapsed().as_millis() as u64,
        });

        // ────────────────────────────────────────────────────────────────────
        // PHASE 5 — REASON
        // ────────────────────────────────────────────────────────────────────
        progress(5, "REASON", "Generating primary response…");
        let p5 = Instant::now();

        let mut messages = vec![
            ChatMessage { role: MessageRole::System, content: system_prompt.clone() },
        ];
        // Include recent context
        for msg in context.iter().rev().take(8).rev() {
            messages.push(msg.clone());
        }
        messages.push(ChatMessage {
            role: MessageRole::User,
            content: enriched_user_message,
        });

        let reason_request = CompletionRequest {
            messages: messages.clone(),
            max_tokens: Some(4096),
            temperature: Some(0.4),
            stream: false,
            task_context: Some(intent.intent_type.to_string()),
        };

        let reason_response = self.brain.router.complete(reason_request).await?;
        let primary_response = reason_response.content.clone();
        let tokens_after_reason = reason_response.input_tokens + reason_response.output_tokens;

        phases.push(PhaseRecord {
            phase: 5, name: "REASON",
            status: PhaseStatus::Ok,
            detail: format!(
                "model={} tokens={} latency={}ms",
                reason_response.model,
                tokens_after_reason,
                reason_response.latency_ms
            ),
            duration_ms: p5.elapsed().as_millis() as u64,
        });

        // ────────────────────────────────────────────────────────────────────
        // PHASE 6 — REFLECT
        // ────────────────────────────────────────────────────────────────────
        progress(6, "REFLECT", "Self-evaluating response quality…");
        let p6 = Instant::now();

        let reflection = self.reflect(&intent, input, &primary_response).await;
        let needs_refine = reflection.needs_refine;
        let reflection_score = reflection.avg_score;

        phases.push(PhaseRecord {
            phase: 6, name: "REFLECT",
            status: if needs_refine { PhaseStatus::Warning } else { PhaseStatus::Ok },
            detail: format!(
                "score={:.1}/10 refine={}{}",
                reflection_score,
                needs_refine,
                if needs_refine {
                    format!(" issues=[{}]", reflection.issues.join(", "))
                } else { String::new() }
            ),
            duration_ms: p6.elapsed().as_millis() as u64,
        });

        // ────────────────────────────────────────────────────────────────────
        // PHASE 7 — REFINE  (conditional)
        // ────────────────────────────────────────────────────────────────────
        let final_response;
        let mut total_tokens = tokens_after_reason;
        let mut final_model = reason_response.model.clone();
        let mut final_provider = reason_response.provider.clone();

        if needs_refine {
            progress(7, "REFINE", &format!("Improving response — {}", reflection.issues.join(", ")));
            let p7 = Instant::now();

            let refine_prompt = format!(
                "Your previous response had these quality issues: {}\n\n\
                 Original question: {}\n\n\
                 Previous response:\n{}\n\n\
                 Rewrite with all issues corrected. Be more complete, structured, and actionable.",
                reflection.issues.join("; "),
                input,
                primary_response
            );

            let refine_request = CompletionRequest {
                messages: vec![
                    ChatMessage { role: MessageRole::System, content: system_prompt },
                    ChatMessage { role: MessageRole::User, content: refine_prompt },
                ],
                max_tokens: Some(4096),
                temperature: Some(0.3),
                stream: false,
                task_context: Some(intent.intent_type.to_string()),
            };

            match self.brain.router.complete(refine_request).await {
                Ok(refined) => {
                    total_tokens += refined.input_tokens + refined.output_tokens;
                    final_model = refined.model.clone();
                    final_provider = refined.provider.clone();
                    final_response = refined.content;
                    phases.push(PhaseRecord {
                        phase: 7, name: "REFINE",
                        status: PhaseStatus::Ok,
                        detail: format!("response improved — {} tokens", total_tokens),
                        duration_ms: p7.elapsed().as_millis() as u64,
                    });
                }
                Err(_) => {
                    // Refine failed — use primary response, not a hard error
                    final_response = primary_response;
                    phases.push(PhaseRecord {
                        phase: 7, name: "REFINE",
                        status: PhaseStatus::Warning,
                        detail: "refinement failed — using primary response".to_string(),
                        duration_ms: p7.elapsed().as_millis() as u64,
                    });
                }
            }
        } else {
            final_response = primary_response;
            phases.push(PhaseRecord {
                phase: 7, name: "REFINE",
                status: PhaseStatus::Skipped,
                detail: format!("skipped — quality score {:.1}/10 is sufficient", reflection_score),
                duration_ms: 0,
            });
        }

        // PHASE 8 — FORMAT
        progress(8, "FORMAT", "Applying agentic output structure…");
        let p8 = Instant::now();
        let formatted = self.format_response(&intent, &final_response, &tools_used, web_result_count);
        phases.push(PhaseRecord {
            phase: 8, name: "FORMAT",
            status: PhaseStatus::Ok,
            detail: format!("output_chars={}", formatted.len()),
            duration_ms: p8.elapsed().as_millis() as u64,
        });

        // ── ReAct Loop — Autonomous command execution ──────────────────────
        let mut commands_run: Vec<shell::ShellResult> = Vec::new();
        let mut post_exec_context = String::new();
        let proposed_cmds = shell::extract_commands_from_ai(&formatted);
        let runnable: Vec<String> = proposed_cmds.into_iter()
            .filter(|cmd| shell::classify_safety(cmd) != shell::CommandSafety::Destructive)
            .take(8)
            .collect();

        if !runnable.is_empty() {
            self.emit(EventKind::Divider, "─".repeat(50));
            self.emit(EventKind::PhaseStart, format!("⚡ Running {} command(s) autonomously", runnable.len()));
            for cmd in &runnable {
                self.emit(EventKind::ToolCall, format!("$ {}", cmd));
                progress(8, "EXECUTE", &format!("$ {}", &cmd[..cmd.len().min(60)]));
                match shell::run(cmd).await {
                    Ok(result) => {
                        self.emit(
                            if result.success { EventKind::Done } else { EventKind::Warning },
                            if result.success {
                                format!("✓ exit 0 · {}ms", result.duration_ms)
                            } else {
                                format!("✗ exit {} · {}ms", result.exit_code, result.duration_ms)
                            },
                        );
                        for line in result.stdout.lines().take(8) {
                            if !line.trim().is_empty() {
                                self.emit(EventKind::Thought, format!("  {}", line));
                            }
                        }
                        if !result.success && !result.stderr.is_empty() {
                            for line in result.stderr.lines().take(4) {
                                self.emit(EventKind::Warning, format!("  {}", line));
                            }
                            // Auto-fix: ask AI what went wrong
                            let fix_prompt = format!(
                                "Command \'{}\' failed (exit {}).\n\nOutput:\n{}\n\nWhat is the exact fix?",
                                result.command, result.exit_code, result.combined_output()
                            );
                            if let Ok(fix) = self.brain.quick_complete(&fix_prompt).await {
                                self.emit(EventKind::PhaseStart, "🔧 Auto-fix analysis:");
                                for line in fix.lines().take(6) {
                                    self.emit(EventKind::Thought, format!("  {}", line));
                                }
                                post_exec_context.push_str(&format!("Fix:\n{}\n\n", fix));
                            }
                        }
                        post_exec_context.push_str(&format!(
                            "$ {}\nexit {}\n{}\n\n",
                            result.command, result.exit_code,
                            result.combined_output().chars().take(2000).collect::<String>()
                        ));
                        if !tools_used.contains(&"shell".to_string()) {
                            tools_used.push("shell".to_string());
                        }
                        commands_run.push(result);
                    }
                    Err(e) => { self.emit(EventKind::Error, format!("Failed: {}", e)); }
                }
            }
            self.emit(EventKind::Divider, "─".repeat(50));
        }

        let final_formatted = if !post_exec_context.is_empty() {
            progress(8, "SYNTHESIZE", "Incorporating command results…");
            let synth = format!(
                "Commands ran:\n{}\n\nSummarise briefly: what succeeded, what failed, what the user should know.",
                post_exec_context
            );
            match self.brain.quick_complete(&synth).await {
                Ok(s) => format!("{}\n\n---\n\n{}", formatted, s),
                Err(_) => formatted,
            }
        } else { formatted };

        // PHASE 9 — COMMIT
        progress(9, "COMMIT", "Persisting to memory…");
        let p9 = Instant::now();
        { let mut mem = self.memory.write().await; let _ = mem.store_interaction(input, &final_formatted).await; }
        phases.push(PhaseRecord {
            phase: 9, name: "COMMIT", status: PhaseStatus::Ok,
            detail: format!("stored · {} cmds · {}ms", commands_run.len(), total_start.elapsed().as_millis()),
            duration_ms: p9.elapsed().as_millis() as u64,
        });
        info!("Pipeline complete — {}ms, {} tokens, {} cmds", total_start.elapsed().as_millis(), total_tokens, commands_run.len());
        Ok(PipelineResult {
            response: final_formatted, model: final_model, provider: final_provider,
            tokens_used: total_tokens, latency_ms: total_start.elapsed().as_millis() as u64,
            phases, tools_used, web_results: web_result_count, commands_run,
        })
    }

    // ─── Phase helpers ────────────────────────────────────────────────────────

    fn build_plan(&self, intent: &ParsedIntent, input: &str) -> Vec<String> {
        match intent.intent_type {
            IntentType::Research => vec![
                "Formulate search queries".to_string(),
                "Search the web for live data".to_string(),
                "Synthesise findings".to_string(),
                "Generate structured report".to_string(),
                "Self-review for completeness".to_string(),
            ],
            IntentType::Code => vec![
                "Analyse requirements".to_string(),
                "Recall relevant patterns from memory".to_string(),
                "Generate complete implementation".to_string(),
                "Add tests and error handling".to_string(),
                "Review for quality".to_string(),
            ],
            IntentType::Business => vec![
                "Identify market/asset".to_string(),
                "Fetch live market intelligence".to_string(),
                "Run technical analysis".to_string(),
                "Assess risk parameters".to_string(),
                "Generate actionable report".to_string(),
            ],
            IntentType::File => vec![
                "Read file contents".to_string(),
                "Analyse structure and content".to_string(),
                "Generate response".to_string(),
            ],
            IntentType::Automate => vec![
                "Identify automation targets".to_string(),
                "Generate complete script".to_string(),
                "Add error handling and logging".to_string(),
                "Provide run instructions".to_string(),
            ],
            _ => {
                let preview = &input[..input.len().min(40)];
                vec![
                    format!("Understand: {}…", preview),
                    "Recall context from memory".to_string(),
                    "Generate comprehensive response".to_string(),
                    "Self-review output".to_string(),
                ]
            }
        }
    }

    fn build_search_queries(&self, intent: &ParsedIntent, input: &str) -> Vec<String> {
        let base = input.trim().to_string();

        match intent.intent_type {
            IntentType::Business => {
                // Extract ticker/crypto symbol
                let symbol = extract_symbol(input).unwrap_or_else(|| "market".to_string());
                vec![
                    format!("{} price today 2026", symbol),
                    format!("{} analysis forecast", symbol),
                ]
            }
            IntentType::Research => {
                vec![
                    base.clone(),
                    format!("{} 2026 latest", base),
                ]
            }
            _ => vec![base],
        }
    }

    fn build_system_prompt(&self, intent: &ParsedIntent, tools_used: &[String]) -> String {
        let tools_str = if tools_used.is_empty() {
            "none".to_string()
        } else {
            tools_used.join(", ")
        };

        format!(
r#"You are MICRODRAGON — a Distributed Intelligence Network operating as a Cognitive Local Autonomous Agent.
Built by EMEMZYVISUALS DIGITALS. Version 0.1.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
IDENTITY & BEHAVIOUR
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
You are NOT a chatbot. You are an autonomous agent.
- You produce COMPLETE, EXECUTABLE outputs — not suggestions
- You NEVER say "I cannot" without offering an alternative path
- You think step-by-step internally, but present conclusions FIRST
- You structure ALL responses with headers, code blocks, lists
- You cite sources [1] [2] when you used web search data
- You end every response with a NEXT STEPS section
- You speak with the authority of an expert, not the caution of a chatbot

CURRENT TASK
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Task type  : {}
Tools used : {}
Mode       : Agentic (9-phase pipeline)

FORMATTING RULES
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
- Always use ## headers to separate major sections
- Always put code in ```language blocks
- Use **bold** for key terms and important findings
- Use numbered lists for steps, bullet lists for options
- Keep paragraphs short (3-4 lines max)
- End with a ## Next Steps or ## Action Items section"#,
            intent.intent_type.to_string().to_uppercase(),
            tools_str
        )
    }

    fn task_preamble(&self, intent: &ParsedIntent) -> String {
        match intent.intent_type {
            IntentType::Research =>
                "Use ALL the web search results below. Synthesise them into a comprehensive, \
                 well-structured research report. Cite sources by number [1], [2], etc.".to_string(),
            IntentType::Business =>
                "Analyse the market data below. Give a concrete trading signal with entry, \
                 stop loss, and target. This is not financial advice — it is agentic analysis.".to_string(),
            IntentType::Code =>
                "Produce complete, production-ready code. No TODOs, no placeholders. \
                 Include all imports, full error handling, and a usage example.".to_string(),
            IntentType::File =>
                "Analyse the file contents below. Give a complete, structured response.".to_string(),
            _ => "Respond completely and actionably to the following.".to_string(),
        }
    }

    // ─── Reflection engine ────────────────────────────────────────────────────

    async fn reflect(&self, intent: &ParsedIntent, input: &str, response: &str) -> ReflectionResult {
        // Use the AI to self-critique — fast, low temperature call
        let reflect_prompt = format!(
            "Quality-check this agent response.\n\
             User asked: \"{}\"\n\
             Task type: {}\n\
             Response:\n---\n{}\n---\n\n\
             Score each dimension 1-10:\n\
             1. COMPLETENESS: Does it fully answer all aspects?\n\
             2. ACTIONABILITY: Can the user immediately act on this?\n\
             3. STRUCTURE: Are headers, code blocks, lists used well?\n\
             4. ACCURACY: Are claims reasonable and well-supported?\n\n\
             Reply ONLY in this exact format:\n\
             SCORES: completeness=X actionability=X structure=X accuracy=X\n\
             ISSUES: [comma-separated list of issues, or NONE]\n\
             VERDICT: APPROVED or REFINE_NEEDED",
            &input[..input.len().min(200)],
            intent.intent_type,
            &response[..response.len().min(1500)]
        );

        match self.brain.quick_complete(&reflect_prompt).await {
            Ok(reflection_text) => parse_reflection(&reflection_text),
            Err(_) => {
                // If reflection fails, approve the primary response
                ReflectionResult {
                    avg_score: 8.0,
                    needs_refine: false,
                    issues: vec![],
                }
            }
        }
    }

    // ─── Output formatter ─────────────────────────────────────────────────────

    fn format_response(
        &self,
        _intent: &ParsedIntent,
        response: &str,
        tools_used: &[String],
        web_results: usize,
    ) -> String {
        // If the response already has good structure (has ##), return as-is
        // The markdown renderer in cli/markdown.rs will handle presentation
        // We just ensure the agentic footer is present

        let mut out = response.to_string();

        // Ensure there's a trailing newline before footer
        if !out.ends_with('\n') {
            out.push('\n');
        }

        // Add provenance footer if web search was used
        if web_results > 0 {
            out.push_str(&format!(
                "\n---\n*🌐 {} web results analysed · Tools: {} · MICRODRAGON v0.1*\n",
                web_results,
                tools_used.join(", ")
            ));
        }

        out
    }
}

// ─── Reflection parsing ───────────────────────────────────────────────────────

struct ReflectionResult {
    avg_score:   f64,
    needs_refine: bool,
    issues:      Vec<String>,
}

fn parse_reflection(text: &str) -> ReflectionResult {
    let text_lower = text.to_lowercase();

    // Parse scores
    let mut scores = Vec::new();
    for dim in &["completeness", "actionability", "structure", "accuracy"] {
        if let Some(val) = extract_score(text, dim) {
            scores.push(val as f64);
        }
    }
    let avg = if scores.is_empty() {
        8.0 // default: approve
    } else {
        scores.iter().sum::<f64>() / scores.len() as f64
    };

    // Parse issues
    let issues = if let Some(issues_line) = text.lines().find(|l| l.starts_with("ISSUES:")) {
        let raw = issues_line.trim_start_matches("ISSUES:").trim();
        if raw == "NONE" || raw.is_empty() {
            vec![]
        } else {
            raw.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
        }
    } else {
        vec![]
    };

    let needs_refine = avg < 7.0 || text_lower.contains("refine_needed");

    ReflectionResult { avg_score: avg, needs_refine, issues }
}

fn extract_score(text: &str, dimension: &str) -> Option<u8> {
    let pattern = format!("{}=", dimension);
    let start = text.to_lowercase().find(&pattern)? + pattern.len();
    let rest = &text[start..];
    let end = rest.find(|c: char| !c.is_ascii_digit()).unwrap_or(rest.len());
    rest[..end].parse::<u8>().ok()
}

// ─── Input analysis helpers ───────────────────────────────────────────────────

fn input_needs_web(input: &str) -> bool {
    let lower = input.to_lowercase();
    let web_triggers = [
        "latest", "current", "today", "now", "recent", "news",
        "price", "stock", "crypto", "btc", "eth", "what happened",
        "who is", "what is the", "2025", "2026", "this week",
        "search", "find out", "look up", "how much is",
    ];
    web_triggers.iter().any(|t| lower.contains(t))
}

fn extract_file_path(input: &str) -> Option<String> {
    // Match patterns like ./file.rs, /path/to/file.py, ~/doc.txt, file.json
    let re = regex::Regex::new(
        r#"(?:^|[\s"])([~/.]?[^\s"]*\.(?:rs|py|js|ts|go|java|c|cpp|h|cs|php|rb|swift|kt|dart|sh|sql|html|css|jsx|tsx|vue|json|toml|yaml|yml|txt|csv|md|env))"#
    ).ok()?;
    let caps = re.captures(input)?;
    Some(caps.get(1)?.as_str().to_string())
}

fn extract_symbol(input: &str) -> Option<String> {
    // Match crypto/stock tickers: BTC, ETH, AAPL, etc.
    let re = regex::Regex::new(r"\b([A-Z]{2,5})\b").ok()?;
    let caps = re.captures(input)?;
    Some(caps.get(1)?.as_str().to_string())
}
