// microdragon-core/src/cli/animation/status.rs
// StatusLine — animated state transitions
// [thinking...] → [planning...] → [executing...] → [done ✓]

use std::io::{self, Write};
use std::time::Instant;
use crossterm::{execute, cursor, style::{Color, Print, ResetColor, SetForegroundColor, Attribute, SetAttribute}, terminal::{Clear, ClearType}};
use crate::cli::terminal;

#[derive(Debug, Clone, PartialEq)]
pub enum StatusState {
    Thinking,
    Planning,
    Executing,
    Fetching,
    Writing,
    Done,
    Error,
    Custom(String),
}

impl StatusState {
    fn label(&self) -> &str {
        match self {
            StatusState::Thinking  => "thinking",
            StatusState::Planning  => "planning",
            StatusState::Executing => "executing",
            StatusState::Fetching  => "fetching",
            StatusState::Writing   => "writing",
            StatusState::Done      => "done",
            StatusState::Error     => "error",
            StatusState::Custom(s) => s.as_str(),
        }
    }

    fn color(&self) -> Color {
        match self {
            StatusState::Thinking | StatusState::Planning => Color::Yellow,
            StatusState::Executing | StatusState::Fetching | StatusState::Writing => Color::Cyan,
            StatusState::Done  => Color::Green,
            StatusState::Error => Color::Red,
            StatusState::Custom(_) => Color::White,
        }
    }

    fn icon(&self, unicode: bool) -> &'static str {
        if unicode {
            match self {
                StatusState::Thinking  => "💭",
                StatusState::Planning  => "📋",
                StatusState::Executing => "⚡",
                StatusState::Fetching  => "🌐",
                StatusState::Writing   => "✍",
                StatusState::Done      => "✓",
                StatusState::Error     => "✗",
                StatusState::Custom(_) => "▸",
            }
        } else {
            match self {
                StatusState::Done  => "[OK]",
                StatusState::Error => "[ERR]",
                _                  => "[..]",
            }
        }
    }
}

pub struct StatusLine {
    current: StatusState,
    start: Instant,
    history: Vec<(StatusState, u64)>,
    is_animated: bool,
}

impl StatusLine {
    pub fn new(initial: StatusState) -> Self {
        let is_animated = super::animations_enabled();
        let mut s = Self {
            current: initial.clone(),
            start: Instant::now(),
            history: Vec::new(),
            is_animated,
        };
        s.render_current();
        s
    }

    /// Transition to a new state
    pub fn transition(&mut self, next: StatusState) {
        let elapsed = self.start.elapsed().as_millis() as u64;
        self.history.push((self.current.clone(), elapsed));
        self.current = next;
        self.render_transition();
    }

    /// Complete with success
    pub fn complete(&mut self, message: &str) {
        self.transition(StatusState::Done);
        if self.is_animated {
            let _ = execute!(
                io::stdout(),
                cursor::MoveToColumn(0),
                Clear(ClearType::CurrentLine),
                SetForegroundColor(Color::Green),
                Print(if terminal::supports_unicode() { "✓ " } else { "[OK] " }),
                ResetColor,
                Print(message),
                Print("\n"),
            );
        } else {
            println!("[OK] {}", message);
        }
    }

    /// Complete with error
    pub fn fail(&mut self, message: &str) {
        self.transition(StatusState::Error);
        if self.is_animated {
            let _ = execute!(
                io::stdout(),
                cursor::MoveToColumn(0),
                Clear(ClearType::CurrentLine),
                SetForegroundColor(Color::Red),
                Print(if terminal::supports_unicode() { "✗ " } else { "[ERR] " }),
                ResetColor,
                Print(message),
                Print("\n"),
            );
        } else {
            println!("[ERR] {}", message);
        }
    }

    fn render_current(&self) {
        if !self.is_animated {
            println!("[{}]...", self.current.label());
            return;
        }

        let icon = self.current.icon(terminal::supports_unicode());
        let label = self.current.label();
        let _ = execute!(
            io::stdout(),
            cursor::MoveToColumn(0),
            Clear(ClearType::CurrentLine),
            SetForegroundColor(self.current.color()),
            Print(icon),
            Print(" "),
            SetAttribute(Attribute::Dim),
            Print(label),
            Print("..."),
            SetAttribute(Attribute::Reset),
            ResetColor,
        );
        let _ = io::stdout().flush();
    }

    fn render_transition(&self) {
        if !self.is_animated {
            println!("[{}]...", self.current.label());
            return;
        }

        // Build a compact breadcrumb trail: thinking → planning → executing...
        let mut trail = String::new();
        for (past_state, _ms) in &self.history {
            trail.push_str(past_state.label());
            trail.push_str(" → ");
        }
        trail.push_str(self.current.label());
        trail.push_str("...");

        let total_elapsed = self.start.elapsed().as_millis();
        let elapsed_str = if total_elapsed > 1000 {
            format!(" ({}s)", total_elapsed / 1000)
        } else {
            String::new()
        };

        let icon = self.current.icon(terminal::supports_unicode());

        let _ = execute!(
            io::stdout(),
            cursor::MoveToColumn(0),
            Clear(ClearType::CurrentLine),
            SetForegroundColor(self.current.color()),
            Print(icon),
            Print(" "),
            SetAttribute(Attribute::Dim),
            Print(&trail),
            SetAttribute(Attribute::Reset),
            SetForegroundColor(Color::DarkGrey),
            Print(&elapsed_str),
            ResetColor,
        );
        let _ = io::stdout().flush();
    }
}

impl Drop for StatusLine {
    fn drop(&mut self) {
        if self.is_animated && self.current != StatusState::Done && self.current != StatusState::Error {
            let _ = execute!(
                io::stdout(),
                cursor::MoveToColumn(0),
                Clear(ClearType::CurrentLine),
            );
            let _ = io::stdout().flush();
        }
    }
}
