// microdragon-core/src/cli/animation/typewriter.rs
// Streaming typewriter effect for AI responses

use std::io::{self, Write};
use std::time::Duration;
use tokio::sync::mpsc::Receiver;
use crossterm::{execute, style::{Color, Print, ResetColor, SetForegroundColor}};
use crate::cli::terminal;

pub struct Typewriter {
    char_delay_ms: u64,
    enabled: bool,
}

impl Typewriter {
    pub fn new() -> Self {
        Self {
            char_delay_ms: 0, // default: no per-char delay — stream tokens as they arrive
            enabled: super::animations_enabled(),
        }
    }

    /// Print a full string with optional typewriter delay
    pub async fn print_animated(&self, text: &str) {
        if !self.enabled || self.char_delay_ms == 0 {
            print!("{}", text);
            let _ = io::stdout().flush();
            return;
        }

        for ch in text.chars() {
            print!("{}", ch);
            let _ = io::stdout().flush();
            if ch == '\n' {
                tokio::time::sleep(Duration::from_millis(self.char_delay_ms * 3)).await;
            } else if ch == '.' || ch == '!' || ch == '?' {
                tokio::time::sleep(Duration::from_millis(self.char_delay_ms * 5)).await;
            } else if ch == ',' {
                tokio::time::sleep(Duration::from_millis(self.char_delay_ms * 2)).await;
            } else {
                tokio::time::sleep(Duration::from_millis(self.char_delay_ms)).await;
            }
        }
    }

    /// Stream tokens from an async channel — the primary streaming output path
    /// Each token is printed immediately as received (true streaming UX)
    pub async fn stream_tokens(&self, mut rx: Receiver<String>) -> String {
        let mut full_output = String::new();

        if terminal::supports_color() {
            let _ = execute!(io::stdout(), SetForegroundColor(Color::White));
        }

        println!(); // blank line before response

        while let Some(token) = rx.recv().await {
            print!("{}", token);
            full_output.push_str(&token);
            let _ = io::stdout().flush();
        }

        if terminal::supports_color() {
            let _ = execute!(io::stdout(), ResetColor);
        }

        println!("\n"); // blank line after response
        full_output
    }

    /// Print a "thinking" prefix before streaming begins
    pub fn print_response_prefix(&self) {
        if terminal::supports_color() {
            let _ = execute!(
                io::stdout(),
                SetForegroundColor(Color::DarkGrey),
                Print("MICRODRAGON ▸ "),
                ResetColor,
            );
        } else {
            print!("MICRODRAGON > ");
        }
        let _ = io::stdout().flush();
    }
}
