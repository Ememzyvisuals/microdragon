// microdragon-core/src/brain/context.rs
// Context Manager - Manages conversation history and context window

use crate::config::providers::{ChatMessage, MessageRole};

pub struct ContextManager {
    max_tokens: usize,
    system_prompt: String,
}

impl ContextManager {
    pub fn new(max_tokens: usize) -> Self {
        Self {
            max_tokens,
            system_prompt: crate::config::DEFAULT_SYSTEM_PROMPT_PLACEHOLDER.to_string(),
        }
    }

    pub fn set_system_prompt(&mut self, prompt: String) {
        self.system_prompt = prompt;
    }

    pub fn build_messages(&self, history: &[ChatMessage], current_input: &str) -> Vec<ChatMessage> {
        let mut messages = vec![
            ChatMessage {
                role: MessageRole::System,
                content: self.system_prompt.clone(),
            }
        ];

        // Add trimmed history
        let trimmed = self.trim_history(history);
        messages.extend(trimmed);

        // Add current user message
        messages.push(ChatMessage {
            role: MessageRole::User,
            content: current_input.to_string(),
        });

        messages
    }

    fn trim_history(&self, history: &[ChatMessage]) -> Vec<ChatMessage> {
        // Estimate tokens: ~4 chars per token
        let system_tokens = self.system_prompt.len() / 4;
        let budget = self.max_tokens.saturating_sub(system_tokens + 2000); // 2000 for response

        let mut total_tokens = 0;
        let mut trimmed = Vec::new();

        // Take from newest to oldest
        for msg in history.iter().rev() {
            let msg_tokens = msg.content.len() / 4 + 10;
            if total_tokens + msg_tokens > budget {
                break;
            }
            total_tokens += msg_tokens;
            trimmed.push(msg.clone());
        }

        trimmed.reverse();
        trimmed
    }

    pub fn estimate_tokens(text: &str) -> usize {
        text.len() / 4
    }
}

// Placeholder to avoid circular import
pub mod defaults {
    pub const SYSTEM_PROMPT: &str = "You are MICRODRAGON, a powerful AI agent. Be precise, helpful, and thorough.";
}
