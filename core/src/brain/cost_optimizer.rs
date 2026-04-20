// microdragon-core/src/brain/cost_optimizer.rs
//
// MICRODRAGON Cost Optimizer
// ────────────────────
// Addresses the #1 pain point: "AI is too expensive"
// Strategies:
//   1. Smart routing — cheap/fast models for simple tasks, powerful for complex
//   2. Response caching — identical prompts return cached answers (free)
//   3. Prompt compression — strip whitespace and redundancy before sending
//   4. Context pruning — remove old irrelevant messages to save context tokens
//   5. Local model fallback — route to Ollama when available (free)
//   6. Token budgeting — warn user before expensive operations

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::config::providers::ModelProvider;
use crate::engine::autonomous::Complexity;

// ─── Cost table (approximate, per 1M tokens, USD) ─────────────────────────────

pub struct ModelCost {
    pub provider: ModelProvider,
    pub model: String,
    pub input_per_1m: f64,
    pub output_per_1m: f64,
    pub context_window: u32,
    pub quality_score: u8,  // 1-10
    pub speed_score: u8,    // 1-10
}

pub fn model_costs() -> Vec<ModelCost> { vec![
    // Free / local
    ModelCost { provider: ModelProvider::Custom, model: "llama3.1:8b".to_string(),  input_per_1m: 0.0, output_per_1m: 0.0, context_window: 8192,   quality_score: 6, speed_score: 8 },
    ModelCost { provider: ModelProvider::Custom, model: "llama3.1:70b".to_string(), input_per_1m: 0.0, output_per_1m: 0.0, context_window: 8192,   quality_score: 8, speed_score: 5 },
    ModelCost { provider: ModelProvider::Custom, model: "qwen2.5:14b".to_string(),  input_per_1m: 0.0, output_per_1m: 0.0, context_window: 32768,  quality_score: 7, speed_score: 7 },
    ModelCost { provider: ModelProvider::Custom, model: "phi4".to_string(),         input_per_1m: 0.0, output_per_1m: 0.0, context_window: 16384,  quality_score: 7, speed_score: 8 },

    // Groq (fastest, very cheap)
    ModelCost { provider: ModelProvider::Groq, model: "llama-3.1-8b-instant".to_string(),   input_per_1m: 0.05, output_per_1m: 0.08,  context_window: 128000, quality_score: 6, speed_score: 10 },
    ModelCost { provider: ModelProvider::Groq, model: "llama-3.3-70b-versatile".to_string(),input_per_1m: 0.59, output_per_1m: 0.79,  context_window: 128000, quality_score: 8, speed_score: 9  },
    ModelCost { provider: ModelProvider::Groq, model: "mixtral-8x7b-32768".to_string(),     input_per_1m: 0.24, output_per_1m: 0.24,  context_window: 32768,  quality_score: 7, speed_score: 9  },

    // OpenRouter (access to cheapest models)
    ModelCost { provider: ModelProvider::OpenRouter, model: "google/gemini-flash-1.5".to_string(), input_per_1m: 0.075, output_per_1m: 0.30, context_window: 1000000, quality_score: 7, speed_score: 9 },
    ModelCost { provider: ModelProvider::OpenRouter, model: "meta-llama/llama-3.1-8b-instruct:free".to_string(), input_per_1m: 0.0, output_per_1m: 0.0, context_window: 131072, quality_score: 6, speed_score: 7 },

    // OpenAI
    ModelCost { provider: ModelProvider::OpenAI, model: "gpt-4o-mini".to_string(),  input_per_1m: 0.15, output_per_1m: 0.60, context_window: 128000, quality_score: 7, speed_score: 9 },
    ModelCost { provider: ModelProvider::OpenAI, model: "gpt-4o".to_string(),       input_per_1m: 2.50, output_per_1m: 10.0, context_window: 128000, quality_score: 9, speed_score: 8 },

    // Anthropic
    ModelCost { provider: ModelProvider::Anthropic, model: "claude-haiku-4-5-20251001".to_string(), input_per_1m: 0.25, output_per_1m: 1.25, context_window: 200000, quality_score: 8, speed_score: 9 },
    ModelCost { provider: ModelProvider::Anthropic, model: "claude-sonnet-4-6".to_string(),         input_per_1m: 3.0,  output_per_1m: 15.0, context_window: 200000, quality_score: 9, speed_score: 8 },
    ModelCost { provider: ModelProvider::Anthropic, model: "claude-opus-4-6".to_string(),           input_per_1m: 15.0, output_per_1m: 75.0, context_window: 200000, quality_score: 10, speed_score: 6 },
] }

// ─── Smart router ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingDecision {
    pub provider: ModelProvider,
    pub model: String,
    pub reason: String,
    pub estimated_cost_usd: f64,
    pub estimated_tokens: u32,
}

pub struct CostOptimizer {
    response_cache: HashMap<u64, CachedResponse>,
    session_spend_usd: f64,
    session_tokens: u64,
    daily_budget_usd: Option<f64>,
    prefer_local: bool,
    prefer_cheap: bool,
}

#[derive(Debug, Clone)]
struct CachedResponse {
    response: String,
    hits: u32,
    created_at: DateTime<Utc>,
    tokens_saved: u64,
}

impl CostOptimizer {
    pub fn new() -> Self {
        let prefer_local = std::env::var("MICRODRAGON_PREFER_LOCAL").unwrap_or_default() == "true"
            || Self::ollama_available();
        let prefer_cheap = std::env::var("MICRODRAGON_PREFER_CHEAP").unwrap_or_default() == "true";
        let daily_budget = std::env::var("MICRODRAGON_DAILY_BUDGET_USD")
            .ok()
            .and_then(|v| v.parse::<f64>().ok());

        Self {
            response_cache: HashMap::new(),
            session_spend_usd: 0.0,
            session_tokens: 0,
            daily_budget_usd: daily_budget,
            prefer_local,
            prefer_cheap,
        }
    }

    /// Check cache before calling AI — free responses for repeated prompts
    pub fn cache_lookup(&self, prompt: &str) -> Option<String> {
        let key = self.hash_prompt(prompt);
        if let Some(cached) = self.response_cache.get(&key) {
            // Cache valid for 1 hour
            let age = Utc::now().signed_duration_since(cached.created_at);
            if age.num_minutes() < 60 {
                debug!("Cache hit! Saved ~{} tokens", cached.tokens_saved);
                return Some(cached.response.clone());
            }
        }
        None
    }

    pub fn cache_store(&mut self, prompt: &str, response: &str, tokens_used: u64) {
        let key = self.hash_prompt(prompt);
        self.response_cache.insert(key, CachedResponse {
            response: response.to_string(),
            hits: 0,
            created_at: Utc::now(),
            tokens_saved: tokens_used,
        });
        // Keep cache to 1000 entries
        if self.response_cache.len() > 1000 {
            let oldest = self.response_cache.iter()
                .min_by_key(|(_, v)| v.created_at)
                .map(|(k, _)| *k);
            if let Some(k) = oldest { self.response_cache.remove(&k); }
        }
    }

    /// Route to cheapest adequate model for this task
    pub fn smart_route(
        &self,
        complexity: &Complexity,
        active_provider: &ModelProvider,
        active_model: &str,
    ) -> RoutingDecision {

        // If user has a local model, always use it for simple tasks
        if self.prefer_local && matches!(complexity, Complexity::Low | Complexity::Medium) {
            return RoutingDecision {
                provider: ModelProvider::Custom,
                model: "llama3.1:8b".to_string().to_string(),
                reason: "Local model (free) for simple task".to_string(),
                estimated_cost_usd: 0.0,
                estimated_tokens: self.estimate_tokens(complexity),
            };
        }

        // If budget-conscious, route simple tasks to cheapest API model
        if self.prefer_cheap {
            match complexity {
                Complexity::Low => {
                    return RoutingDecision {
                        provider: ModelProvider::Groq,
                        model: "llama-3.1-8b-instant".to_string().to_string(),
                        reason: "Groq instant (fastest, cheapest) for simple task".to_string(),
                        estimated_cost_usd: self.estimate_cost(Complexity::Low, 0.05, 0.08),
                        estimated_tokens: self.estimate_tokens(complexity),
                    };
                }
                Complexity::Medium => {
                    return RoutingDecision {
                        provider: ModelProvider::Groq,
                        model: "llama-3.3-70b-versatile".to_string().to_string(),
                        reason: "Groq 70B for medium task (good quality, low cost)".to_string(),
                        estimated_cost_usd: self.estimate_cost(Complexity::Medium, 0.59, 0.79),
                        estimated_tokens: self.estimate_tokens(complexity),
                    };
                }
                _ => {}
            }
        }

        // Default: use configured provider
        let tokens = self.estimate_tokens(complexity);
        let cost = self.estimate_cost_for_model(active_model, tokens);

        RoutingDecision {
            provider: active_provider.clone(),
            model: active_model.to_string(),
            reason: "Using configured provider".to_string(),
            estimated_cost_usd: cost,
            estimated_tokens: tokens,
        }
    }

    /// Compress a prompt to reduce token usage
    pub fn compress_prompt(prompt: &str) -> String {
        let mut compressed = prompt.to_string();

        // Remove excessive whitespace
        let ws_re = regex::Regex::new(r"\n{3,}").unwrap();
        compressed = ws_re.replace_all(&compressed, "\n\n").to_string();

        let space_re = regex::Regex::new(r"[ \t]{2,}").unwrap();
        compressed = space_re.replace_all(&compressed, " ").to_string();

        // Remove common filler phrases that don't add context
        let fillers = [
            "As an AI language model, ",
            "I am an AI assistant, ",
            "Certainly! ",
            "Of course! ",
            "Sure! ",
            "Absolutely! ",
        ];
        for filler in &fillers {
            compressed = compressed.replace(filler, "");
        }

        compressed.trim().to_string()
    }

    /// Prune conversation context to stay within budget
    pub fn prune_context(
        messages: &[crate::config::providers::ChatMessage],
        max_tokens: usize,
    ) -> Vec<crate::config::providers::ChatMessage> {
        // Always keep system message + last 4 exchanges
        let total_chars: usize = messages.iter().map(|m| m.content.len()).sum();
        let approx_tokens = total_chars / 4;

        if approx_tokens <= max_tokens {
            return messages.to_vec();
        }

        // Keep first message (system), last 8 messages
        let mut pruned = Vec::new();
        if let Some(first) = messages.first() {
            pruned.push(first.clone());
        }
        let tail_start = messages.len().saturating_sub(8);
        pruned.extend_from_slice(&messages[tail_start.max(1)..]);

        debug!("Context pruned: {} → {} messages to save tokens", messages.len(), pruned.len());
        pruned
    }

    /// Budget check — warn if approaching daily limit
    pub fn check_budget(&self, estimated_cost: f64) -> BudgetStatus {
        if let Some(budget) = self.daily_budget_usd {
            let after = self.session_spend_usd + estimated_cost;
            if after > budget {
                return BudgetStatus::Exceeded { spent: self.session_spend_usd, budget };
            }
            if after > budget * 0.8 {
                return BudgetStatus::NearLimit { spent: self.session_spend_usd, budget, remaining: budget - self.session_spend_usd };
            }
        }
        BudgetStatus::Ok
    }

    pub fn record_spend(&mut self, tokens_used: u32, model: &str) {
        let cost = self.estimate_cost_for_model(model, tokens_used);
        self.session_spend_usd += cost;
        self.session_tokens += tokens_used as u64;
    }

    pub fn session_summary(&self) -> CostSummary {
        CostSummary {
            session_tokens: self.session_tokens,
            session_cost_usd: self.session_spend_usd,
            cache_entries: self.response_cache.len(),
            prefer_local: self.prefer_local,
        }
    }

    // ── Helpers ───────────────────────────────────────────────────────────────

    fn hash_prompt(&self, prompt: &str) -> u64 {
        let mut h = DefaultHasher::new();
        prompt.trim().hash(&mut h);
        h.finish()
    }

    fn estimate_tokens(&self, complexity: &Complexity) -> u32 {
        match complexity {
            Complexity::Low    => 300,
            Complexity::Medium => 1000,
            Complexity::High   => 3000,
            Complexity::Epic   => 8000,
        }
    }

    fn estimate_cost(&self, complexity: Complexity, input_rate: f64, output_rate: f64) -> f64 {
        let tokens = self.estimate_tokens(&complexity) as f64;
        let input_tokens = tokens * 0.6;
        let output_tokens = tokens * 0.4;
        (input_tokens * input_rate / 1_000_000.0) + (output_tokens * output_rate / 1_000_000.0)
    }

    fn estimate_cost_for_model(&self, model: &str, tokens: u32) -> f64 {
        for mc in MODEL_COSTS {
            if mc.model == model {
                let input = tokens as f64 * 0.6 * mc.input_per_1m / 1_000_000.0;
                let output = tokens as f64 * 0.4 * mc.output_per_1m / 1_000_000.0;
                return input + output;
            }
        }
        0.0 // Unknown model
    }

    fn ollama_available() -> bool {
        // Quick check: is Ollama process running?
        std::process::Command::new("ollama")
            .args(["list"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

#[derive(Debug, Clone)]
pub enum BudgetStatus {
    Ok,
    NearLimit { spent: f64, budget: f64, remaining: f64 },
    Exceeded  { spent: f64, budget: f64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostSummary {
    pub session_tokens: u64,
    pub session_cost_usd: f64,
    pub cache_entries: usize,
    pub prefer_local: bool,
}
