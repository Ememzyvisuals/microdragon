// microdragon-core/src/engine/agents.rs
//
// MICRODRAGON Agent Hierarchy System
// ────────────────────────────
// Master agent delegates to specialised sub-agents.
// Each agent stays in its role — no uncontrolled spawning.
// This beats Claude Code (single agent) and OpenClaw (no typed hierarchy).

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
use uuid::Uuid;

use crate::engine::autonomous::{AgentRole, PipelineTask, PipelinePhase};

// ─── Agent definition ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub role: AgentRole,
    pub name: String,
    pub system_prompt: String,
    pub max_tokens: u32,
    pub temperature: f64,
    pub allowed_tools: Vec<String>,
    pub enabled: bool,
}

impl AgentConfig {
    pub fn for_role(role: AgentRole) -> Self {
        match role {
            AgentRole::Master => Self {
                role,
                name: "MICRODRAGON Master".to_string(),
                system_prompt: MASTER_PROMPT.to_string(),
                max_tokens: 8192,
                temperature: 0.7,
                allowed_tools: vec!["all".to_string()],
                enabled: true,
            },
            AgentRole::Coding => Self {
                role,
                name: "MICRODRAGON Coder".to_string(),
                system_prompt: CODING_PROMPT.to_string(),
                max_tokens: 4096,
                temperature: 0.2,
                allowed_tools: vec!["file_read".to_string(), "file_write".to_string(),
                                    "shell_safe".to_string(), "git".to_string()],
                enabled: true,
            },
            AgentRole::Research => Self {
                role,
                name: "MICRODRAGON Researcher".to_string(),
                system_prompt: RESEARCH_PROMPT.to_string(),
                max_tokens: 4096,
                temperature: 0.4,
                allowed_tools: vec!["web_search".to_string(), "web_fetch".to_string()],
                enabled: true,
            },
            AgentRole::Business => Self {
                role,
                name: "MICRODRAGON Analyst".to_string(),
                system_prompt: BUSINESS_PROMPT.to_string(),
                max_tokens: 4096,
                temperature: 0.3,
                allowed_tools: vec!["market_data".to_string(), "calculator".to_string()],
                enabled: true,
            },
            AgentRole::Automation => Self {
                role,
                name: "MICRODRAGON Automator".to_string(),
                system_prompt: AUTOMATION_PROMPT.to_string(),
                max_tokens: 2048,
                temperature: 0.1,
                allowed_tools: vec!["playwright".to_string(), "pyautogui".to_string(),
                                    "screenshot".to_string()],
                enabled: true,
            },
            AgentRole::Writing => Self {
                role,
                name: "MICRODRAGON Writer".to_string(),
                system_prompt: WRITING_PROMPT.to_string(),
                max_tokens: 4096,
                temperature: 0.8,
                allowed_tools: vec!["file_write".to_string()],
                enabled: true,
            },
            AgentRole::Security => Self {
                role,
                name: "MICRODRAGON Security".to_string(),
                system_prompt: SECURITY_PROMPT.to_string(),
                max_tokens: 4096,
                temperature: 0.1,
                allowed_tools: vec!["file_read".to_string(), "static_analysis".to_string()],
                enabled: true,
            },
        }
    }
}

// ─── Agent registry ───────────────────────────────────────────────────────────

pub struct AgentRegistry {
    agents: HashMap<String, AgentConfig>,
}

impl AgentRegistry {
    pub fn new() -> Self {
        let mut agents = HashMap::new();
        for role in [
            AgentRole::Master, AgentRole::Coding, AgentRole::Research,
            AgentRole::Business, AgentRole::Automation, AgentRole::Writing,
            AgentRole::Security,
        ] {
            let cfg = AgentConfig::for_role(role.clone());
            agents.insert(role.to_string(), cfg);
        }
        info!("Agent registry: {} agents registered", agents.len());
        Self { agents }
    }

    pub fn get(&self, role: &AgentRole) -> Option<&AgentConfig> {
        self.agents.get(&role.to_string())
    }

    pub fn list_enabled(&self) -> Vec<&AgentConfig> {
        self.agents.values().filter(|a| a.enabled).collect()
    }

    pub fn route_task(&self, input: &str) -> AgentRole {
        let lower = input.to_lowercase();

        // Coding patterns
        if lower.contains("code") || lower.contains("function") || lower.contains("debug")
            || lower.contains("script") || lower.contains("program") || lower.contains("fix")
            || lower.contains("python") || lower.contains("rust") || lower.contains("javascript")
            || lower.contains("typescript") || lower.contains("api") || lower.contains("class")
            || lower.contains("test") && (lower.contains("write") || lower.contains("unit"))
        {
            return AgentRole::Coding;
        }

        // Research patterns
        if lower.contains("research") || lower.contains("search") || lower.contains("find")
            || lower.contains("what is") || lower.contains("explain") || lower.contains("summarize")
            || lower.contains("news") || lower.contains("latest") || lower.contains("look up")
        {
            return AgentRole::Research;
        }

        // Business / finance
        if lower.contains("market") || lower.contains("stock") || lower.contains("trade")
            || lower.contains("crypto") || lower.contains("invest") || lower.contains("portfolio")
            || lower.contains("revenue") || lower.contains("profit") || lower.contains("forecast")
        {
            return AgentRole::Business;
        }

        // Automation
        if lower.contains("automate") || lower.contains("browser") || lower.contains("click")
            || lower.contains("screenshot") || lower.contains("fill") || lower.contains("scrape")
            || lower.contains("download") || lower.contains("desktop") || lower.contains("run script")
        {
            return AgentRole::Automation;
        }

        // Writing / creative
        if lower.contains("write") || lower.contains("draft") || lower.contains("email")
            || lower.contains("letter") || lower.contains("essay") || lower.contains("article")
            || lower.contains("blog") || lower.contains("story") || lower.contains("poem")
        {
            return AgentRole::Writing;
        }

        // Security
        if lower.contains("security") || lower.contains("vulnerability") || lower.contains("audit")
            || lower.contains("pentest") || lower.contains("exploit") || lower.contains("review")
               && lower.contains("code") && lower.contains("safe")
        {
            return AgentRole::Security;
        }

        // Default to master for complex or unclear tasks
        AgentRole::Master
    }
}

// ─── Agent system prompts ─────────────────────────────────────────────────────

const MASTER_PROMPT: &str = "You are the MICRODRAGON Master Agent — the central orchestrator. \
You analyze complex tasks, delegate to specialist agents, and synthesize their results. \
When a task requires multiple specializations, you break it into subtasks and coordinate. \
Always think step-by-step. Always plan before acting. Never execute without analysis.";

const CODING_PROMPT: &str = "You are MICRODRAGON Coder — a world-class software engineer. \
You write production-grade code with proper error handling, tests, and documentation. \
Languages: Python, Rust, JavaScript, TypeScript, Go, Bash, and more. \
Always include: imports, error handling, edge cases, usage examples. \
Output working code only — no placeholder functions, no TODO comments left unfilled.";

const RESEARCH_PROMPT: &str = "You are MICRODRAGON Researcher — a thorough intelligence analyst. \
You synthesize information from multiple sources into clear, structured reports. \
Always cite sources, indicate confidence levels, distinguish facts from opinions. \
Format: Executive Summary → Key Findings → Evidence → Conclusions → Sources.";

const BUSINESS_PROMPT: &str = "You are MICRODRAGON Analyst — a senior financial and business analyst. \
You provide data-driven market analysis, risk assessment, and actionable trading insights. \
Always include: data source, time period, confidence interval, key risks, disclaimer. \
NEVER provide financial advice without explicit risk disclaimer.";

const AUTOMATION_PROMPT: &str = "You are MICRODRAGON Automator — a precision automation engineer. \
You create safe, reliable automation scripts using Playwright and PyAutoGUI. \
Always include: FAILSAFE = True, error handling, timeouts, safety delays. \
NEVER write scripts that delete files, send emails, or modify system settings without explicit user approval.";

const WRITING_PROMPT: &str = "You are MICRODRAGON Writer — a professional writer and communicator. \
You craft compelling, clear, and appropriate content for any context. \
Always match tone to purpose: professional for business, creative for fiction, concise for email. \
Output complete, ready-to-use content — not outlines unless specifically requested.";

const SECURITY_PROMPT: &str = "You are MICRODRAGON Security — a senior cybersecurity expert. \
You review code and systems for vulnerabilities, misconfigurations, and security risks. \
Always explain: severity (CRITICAL/HIGH/MEDIUM/LOW), attack vector, remediation steps. \
NEVER provide working exploit code. Focus on defense and remediation.";
