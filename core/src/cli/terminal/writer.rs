// microdragon-core/src/cli/terminal/writer.rs
// TermWriter — safe cross-platform terminal output
// Automatically degrades from rich ANSI to plain text

use std::io::{self, Write};
use crossterm::{
    execute, queue,
    style::{Color, Print, ResetColor, SetForegroundColor, Attribute, SetAttribute},
    cursor::{Hide, Show, MoveToColumn, MoveUp},
    terminal::{Clear, ClearType},
};
use super::caps::TermLevel;
use super::caps;

pub struct TermWriter {
    pub caps: std::sync::Arc<super::caps::TermCaps>,
}

impl TermWriter {
    pub fn new() -> Self {
        Self { caps: super::super::terminal::caps() }
    }

    // ─── Core print primitives ─────────────────────────────────────────────

    /// Print plain text (always works)
    pub fn print(&self, text: &str) {
        print!("{}", text);
        let _ = io::stdout().flush();
    }

    pub fn println(&self, text: &str) {
        println!("{}", text);
    }

    /// Print with color — falls back to plain on Simple/None terminals
    pub fn print_color(&self, text: &str, color: TermColor) {
        if self.caps.ansi_color {
            let _ = execute!(
                io::stdout(),
                SetForegroundColor(color.to_crossterm()),
                Print(text),
                ResetColor,
            );
        } else {
            print!("{}", text);
        }
        let _ = io::stdout().flush();
    }

    pub fn println_color(&self, text: &str, color: TermColor) {
        self.print_color(text, color);
        println!();
    }

    pub fn success(&self, text: &str) {
        let prefix = if self.caps.unicode { "✓ " } else { "[OK] " };
        self.println_color(&format!("{}{}", prefix, text), TermColor::Green);
    }

    pub fn error(&self, text: &str) {
        let prefix = if self.caps.unicode { "✗ " } else { "[ERR] " };
        self.println_color(&format!("{}{}", prefix, text), TermColor::Red);
    }

    pub fn warn(&self, text: &str) {
        let prefix = if self.caps.unicode { "⚠ " } else { "[WARN] " };
        self.println_color(&format!("{}{}", prefix, text), TermColor::Yellow);
    }

    pub fn info(&self, text: &str) {
        let prefix = if self.caps.unicode { "ℹ " } else { "[INFO] " };
        self.println_color(&format!("{}{}", prefix, text), TermColor::Cyan);
    }

    pub fn dim(&self, text: &str) {
        if self.caps.ansi_color {
            let _ = execute!(
                io::stdout(),
                SetAttribute(Attribute::Dim),
                Print(text),
                SetAttribute(Attribute::Reset),
            );
        } else {
            print!("{}", text);
        }
    }

    pub fn bold(&self, text: &str) {
        if self.caps.ansi_color {
            let _ = execute!(
                io::stdout(),
                SetAttribute(Attribute::Bold),
                Print(text),
                SetAttribute(Attribute::Reset),
            );
        } else {
            print!("{}", text);
        }
    }

    /// Print a horizontal rule
    pub fn rule(&self, title: &str) {
        let width = self.caps.width as usize;
        if title.is_empty() {
            let line = "─".repeat(width.min(60));
            self.println_color(&line, TermColor::DimWhite);
        } else {
            let pad = (width.min(60).saturating_sub(title.len() + 4)) / 2;
            let rule = format!("{} {} {}", "─".repeat(pad), title, "─".repeat(pad));
            self.println_color(&rule, TermColor::Cyan);
        }
    }

    /// Print a section header
    pub fn header(&self, text: &str) {
        println!();
        if self.caps.ansi_color {
            let _ = execute!(
                io::stdout(),
                SetForegroundColor(Color::Cyan),
                SetAttribute(Attribute::Bold),
                Print(text),
                SetAttribute(Attribute::Reset),
                ResetColor,
                Print("\n"),
            );
        } else {
            println!("=== {} ===", text);
        }
    }

    /// Print key-value pair
    pub fn kv(&self, key: &str, value: &str) {
        if self.caps.ansi_color {
            let _ = execute!(
                io::stdout(),
                SetForegroundColor(Color::DarkGrey),
                Print(format!("  {:20}", key)),
                ResetColor,
                SetForegroundColor(Color::White),
                Print(value),
                ResetColor,
                Print("\n"),
            );
        } else {
            println!("  {:20} {}", key, value);
        }
    }

    // ─── Cursor operations (rich only) ────────────────────────────────────

    pub fn hide_cursor(&self) {
        if self.caps.cursor_movement {
            let _ = execute!(io::stdout(), Hide);
        }
    }

    pub fn show_cursor(&self) {
        if self.caps.cursor_movement {
            let _ = execute!(io::stdout(), Show);
        }
    }

    pub fn clear_line(&self) {
        if self.caps.cursor_movement {
            let _ = execute!(
                io::stdout(),
                MoveToColumn(0),
                Clear(ClearType::CurrentLine),
            );
        }
    }

    pub fn move_up(&self, lines: u16) {
        if self.caps.cursor_movement {
            let _ = execute!(io::stdout(), MoveUp(lines));
        }
    }

    pub fn flush(&self) {
        let _ = io::stdout().flush();
    }
}

/// Color abstraction that maps to crossterm colors
#[derive(Debug, Clone, Copy)]
pub enum TermColor {
    Green,
    Red,
    Yellow,
    Cyan,
    Blue,
    Magenta,
    White,
    DimWhite,
    Orange,
}

impl TermColor {
    pub fn to_crossterm(self) -> Color {
        match self {
            TermColor::Green => Color::Green,
            TermColor::Red => Color::Red,
            TermColor::Yellow => Color::Yellow,
            TermColor::Cyan => Color::Cyan,
            TermColor::Blue => Color::Blue,
            TermColor::Magenta => Color::Magenta,
            TermColor::White => Color::White,
            TermColor::DimWhite => Color::DarkGrey,
            TermColor::Orange => Color::Rgb { r: 255, g: 140, b: 0 },
        }
    }
}
