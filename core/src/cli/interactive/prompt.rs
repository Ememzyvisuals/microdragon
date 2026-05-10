// microdragon-core/src/cli/interactive/prompt.rs
// Prompt rendering — builds the input prompt string based on terminal caps

use crate::cli::terminal;
use crossterm::style::{Color, Stylize};

pub struct PromptRenderer {
    turn: u32,
}

impl PromptRenderer {
    pub fn new() -> Self { Self { turn: 0 } }

    pub fn next_turn(&mut self) { self.turn += 1; }

    pub fn render(&self) -> String {
        if terminal::is_rich() && terminal::supports_color() {
            format!("\n {} {} ",
                "⬡".with(Color::Cyan).bold(),
                format!("#{}", self.turn).with(Color::DarkGrey),
            )
        } else {
            format!("\nmicrodragon[{}]> ", self.turn)
        }
    }

    pub fn continuation() -> String {
        if terminal::supports_color() {
            "  … ".with(Color::DarkGrey).to_string()
        } else {
            "  ... ".to_string()
        }
    }
}
