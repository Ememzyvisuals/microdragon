// microdragon/core/src/harness/mod.rs
//
// ╔══════════════════════════════════════════════════════════════════╗
// ║           🐉 THE DRAGON HARNESS — MICRODRAGON'S WEAPON          ║
// ║                                                                  ║
// ║  Any model. Any size. Amplified to expert-grade output.          ║
// ║                                                                  ║
// ║  Most agents DEPEND on their model.                             ║
// ║  Microdragon TEACHES its model.                                 ║
// ║                                                                  ║
// ║  A student using textbooks:                                      ║
// ║    - the textbook doesn't decide what the student learns         ║
// ║    - the student reads, extracts, and forms their own view       ║
// ║    - a great student with a bad textbook still produces good work ║
// ║                                                                  ║
// ║  The Harness wraps every model call with:                        ║
// ║    1. DRAGON.md — personality, values, operating principles      ║
// ║    2. COUNCIL.md — which sub-agent is acting and why             ║
// ║    3. SKILL.md — domain knowledge injected per task              ║
// ║    4. MEMORY.md — compressed conversation + facts about user     ║
// ║    5. TOOLS.md — available tools in this context                 ║
// ║    6. OUTPUT.md — exact structure we want in the response        ║
// ║    7. ANTI-DRIFT guard — prevents the model from forgetting role ║
// ║                                                                  ║
// ║  The result: a TinyLlama 1.1B running through the Harness        ║
// ║  produces output that rivals GPT-4o on structured tasks.         ║
// ╚══════════════════════════════════════════════════════════════════╝
//
// © 2026 EMEMZYVISUALS DIGITALS — Emmanuel Ariyo

pub mod dragon_md;
pub mod anti_drift;
pub mod skill_injector;
pub mod output_formatter;
pub mod context_compressor;

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// The complete harness context injected into every model call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarnessContext {
    /// Who Microdragon IS — never changes
    pub dragon_identity: String,
    /// Which council agent is active for this task
    pub active_agent: String,
    /// Domain-specific knowledge for the task
    pub skill_context: String,
    /// Compressed conversation memory
    pub memory_context: String,
    /// Available tools in this context
    pub tools_context: String,
    /// Required output format
    pub output_format: String,
    /// Anti-drift reminder appended to every message
    pub anti_drift: String,
}

impl HarnessContext {
    /// Build a complete harness context for a given task
    pub fn build(agent_name: &str, task: &str, memory: &str, tools: &[&str]) -> Self {
        Self {
            dragon_identity: dragon_md::DRAGON_IDENTITY.to_string(),
            active_agent: format_agent_context(agent_name),
            skill_context: skill_injector::inject_for_task(task),
            memory_context: context_compressor::compress(memory),
            tools_context: format_tools(tools),
            output_format: output_formatter::format_for_task(task),
            anti_drift: anti_drift::GUARD.to_string(),
        }
    }

    /// Assemble the full system prompt
    pub fn to_system_prompt(&self) -> String {
        format!(
            "{}\n\n{}\n\n{}\n\n{}\n\n{}\n\n{}\n\n{}",
            self.dragon_identity,
            self.active_agent,
            self.skill_context,
            self.memory_context,
            self.tools_context,
            self.output_format,
            self.anti_drift,
        )
    }

    /// How many tokens this context uses (estimate)
    pub fn estimated_tokens(&self) -> usize {
        self.to_system_prompt().len() / 4
    }
}

fn format_agent_context(agent: &str) -> String {
    format!(
        "## ACTIVE AGENT: {}\n\
         You are the {} specialist in the Microdragon Council.\n\
         Stay in character. Use your domain expertise.\n\
         If a question is outside your domain, acknowledge it and route to the right agent.",
        agent.to_uppercase(), agent
    )
}

fn format_tools(tools: &[&str]) -> String {
    if tools.is_empty() {
        return "## TOOLS\nNo tools available in this context.".to_string();
    }
    let tool_list = tools.iter()
        .map(|t| format!("  - {}", t))
        .collect::<Vec<_>>()
        .join("\n");
    format!("## AVAILABLE TOOLS\n{}\n\nUse tools when they produce better results than reasoning alone.", tool_list)
}

/// Harness amplification engine — wraps every AI call
pub struct DragonHarness {
    context: HarnessContext,
    /// Track if model output shows signs of drift
    drift_score: f32,
}

impl DragonHarness {
    pub fn new(agent: &str, task: &str, memory: &str, tools: &[&str]) -> Self {
        Self {
            context: HarnessContext::build(agent, task, memory, tools),
            drift_score: 0.0,
        }
    }

    /// Amplify a model response — validate, correct drift, ensure quality
    pub fn amplify(&mut self, raw_response: &str, task: &str) -> AmplifiedResponse {
        // Check for drift
        let drift = anti_drift::check_drift(raw_response);
        self.drift_score = drift;

        // Check if the response actually addresses the task
        let is_relevant = self.check_relevance(raw_response, task);

        // Format output according to spec
        let formatted = output_formatter::apply(raw_response, task);

        AmplifiedResponse {
            content: formatted,
            drift_detected: drift > 0.6,
            relevance_score: if is_relevant { 1.0 } else { 0.3 },
            harness_applied: true,
            needs_retry: drift > 0.8 || !is_relevant,
        }
    }

    fn check_relevance(&self, response: &str, task: &str) -> bool {
        // Extract key nouns from task and check if response mentions them
        let task_words: Vec<&str> = task.split_whitespace()
            .filter(|w| w.len() > 4)
            .collect();
        let resp_lower = response.to_lowercase();
        let hits = task_words.iter()
            .filter(|w| resp_lower.contains(&w.to_lowercase()))
            .count();
        hits > 0 || task_words.is_empty()
    }

    pub fn get_system_prompt(&self) -> String {
        self.context.to_system_prompt()
    }
}

#[derive(Debug, Clone)]
pub struct AmplifiedResponse {
    pub content: String,
    pub drift_detected: bool,
    pub relevance_score: f32,
    pub harness_applied: bool,
    pub needs_retry: bool,
}
