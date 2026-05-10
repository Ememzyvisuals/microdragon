// microdragon-core/src/cli/animation/progress.rs
// Progress bar — cross-platform, CMD-safe

use std::io::{self, Write};
use std::time::Instant;
use crossterm::{execute, cursor, style::{Color, Print, ResetColor, SetForegroundColor}, terminal::{Clear, ClearType}};
use crate::cli::terminal;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProgressStyle {
    /// ████████░░░░ filled blocks (Unicode)
    Block,
    /// ========----  ASCII (CMD-safe)
    Ascii,
    /// ▓▓▓▓░░░░  half-blocks
    Half,
    /// Auto-select based on terminal
    Auto,
}

impl ProgressStyle {
    pub fn resolve(self) -> Self {
        if self == ProgressStyle::Auto {
            if terminal::supports_unicode() && terminal::is_rich() {
                ProgressStyle::Block
            } else {
                ProgressStyle::Ascii
            }
        } else {
            self
        }
    }

    fn filled_char(&self) -> &'static str {
        match self {
            ProgressStyle::Block => "█",
            ProgressStyle::Ascii => "=",
            ProgressStyle::Half  => "▓",
            _ => "=",
        }
    }

    fn empty_char(&self) -> &'static str {
        match self {
            ProgressStyle::Block => "░",
            ProgressStyle::Ascii => "-",
            ProgressStyle::Half  => "░",
            _ => "-",
        }
    }
}

pub struct ProgressBar {
    total: u64,
    current: u64,
    style: ProgressStyle,
    label: String,
    width: usize,
    start: Instant,
    is_animated: bool,
}

impl ProgressBar {
    pub fn new(total: u64, label: impl Into<String>) -> Self {
        let is_animated = super::animations_enabled();
        let caps = terminal::caps();
        let width = (caps.width as usize).min(60).saturating_sub(30);

        if is_animated {
            let _ = execute!(io::stdout(), cursor::Hide);
        }

        Self {
            total,
            current: 0,
            style: ProgressStyle::Auto,
            label: label.into(),
            width,
            start: Instant::now(),
            is_animated,
        }
    }

    pub fn with_style(mut self, style: ProgressStyle) -> Self {
        self.style = style;
        self
    }

    /// Update progress to an absolute value
    pub fn set(&mut self, value: u64) {
        self.current = value.min(self.total);
        self.render();
    }

    /// Increment progress by a delta
    pub fn inc(&mut self, delta: u64) {
        self.current = (self.current + delta).min(self.total);
        self.render();
    }

    /// Finish the progress bar
    pub fn finish(&mut self, msg: impl Into<String>) {
        self.current = self.total;
        self.render();
        if self.is_animated {
            println!();
            let _ = execute!(io::stdout(), cursor::Show);
        }
        let m = msg.into();
        if !m.is_empty() {
            let prefix = if terminal::supports_unicode() { "✓ " } else { "[OK] " };
            if terminal::supports_color() {
                let _ = execute!(
                    io::stdout(),
                    SetForegroundColor(Color::Green),
                    Print(format!("{}{}\n", prefix, m)),
                    ResetColor,
                );
            } else {
                println!("{}{}", prefix, m);
            }
        }
    }

    fn render(&self) {
        if !self.is_animated { return; }

        let style = self.style.resolve();
        let pct = if self.total == 0 { 1.0 } else { self.current as f64 / self.total as f64 };
        let filled = (pct * self.width as f64).round() as usize;
        let empty = self.width.saturating_sub(filled);

        let bar = format!(
            "{}{}",
            style.filled_char().repeat(filled),
            style.empty_char().repeat(empty)
        );

        let elapsed = self.start.elapsed().as_secs_f64();
        let eta = if pct > 0.01 && pct < 1.0 {
            let total_est = elapsed / pct;
            let remaining = (total_est - elapsed) as u64;
            format!(" ETA {}s", remaining)
        } else {
            String::new()
        };

        let line = format!(
            " {:20} [{}] {:5.1}%{}",
            &self.label[..self.label.len().min(20)],
            bar,
            pct * 100.0,
            eta
        );

        let _ = execute!(
            io::stdout(),
            cursor::MoveToColumn(0),
            Clear(ClearType::CurrentLine),
            SetForegroundColor(if pct >= 1.0 { Color::Green } else { Color::Cyan }),
            Print(&line),
            ResetColor,
        );
        let _ = io::stdout().flush();
    }
}

impl Drop for ProgressBar {
    fn drop(&mut self) {
        if self.is_animated {
            let _ = execute!(io::stdout(), cursor::Show);
        }
    }
}
