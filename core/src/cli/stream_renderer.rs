// microdragon-core/src/cli/stream_renderer.rs
//
// Streaming Output Renderer
// ──────────────────────────
// Handles the real-time display of AI response tokens as they stream in.
// Supports three rendering modes:
//   Rich     — typewriter effect with buffered newlines, word-wrap aware
//   Standard — direct flush on every token with minimal formatting
//   Plain    — line-buffered, prints complete lines only

use std::io::{self, Write};
use std::time::Duration;
use crossterm::{
    style::{Color, Stylize},
    cursor, terminal,
    execute, queue,
};
use tokio::sync::mpsc::Receiver;
use crate::cli::terminal::CAPS;
use crate::cli::theme::Theme;

// ─── StreamRenderer ──────────────────────────────────────────────────────────

pub struct StreamRenderer {
    /// Current column cursor (for word-wrap)
    col: usize,
    /// Internal line buffer (Plain mode only)
    line_buf: String,
    /// Whether the header ("MICRODRAGON:" label) has been printed
    header_printed: bool,
    /// Accumulated full response for return value
    full_response: String,
}

impl StreamRenderer {
    pub fn new() -> Self {
        Self {
            col: 0,
            line_buf: String::new(),
            header_printed: false,
            full_response: String::new(),
        }
    }

    /// Consume a streaming receiver and render tokens as they arrive.
    /// Returns the complete response string when the stream ends.
    pub async fn render_stream(&mut self, mut rx: Receiver<String>) -> String {
        self.print_header();

        while let Some(token) = rx.recv().await {
            self.render_token(&token);
        }

        self.finish();
        self.full_response.clone()
    }

    fn print_header(&mut self) {
        if self.header_printed { return; }
        self.header_printed = true;

        let mut stdout = io::stdout();
        println!(); // blank line before response

        if CAPS.ansi {
            let label = format!(" {} ", Theme::glyph_microdragon()).with(Color::Cyan).bold();
            let _ = write!(stdout, "{}\n", label);
            let _ = write!(stdout, "{}\n", Theme::separator(CAPS.width.saturating_sub(2) as usize));
        } else {
            let _ = writeln!(stdout, "\n[MICRODRAGON]");
            let _ = writeln!(stdout, "{}", "-".repeat(60));
        }

        let _ = stdout.flush();
    }

    fn render_token(&mut self, token: &str) {
        self.full_response.push_str(token);

        match CAPS.profile {
            crate::cli::terminal::DisplayProfile::Rich => self.render_rich(token),
            crate::cli::terminal::DisplayProfile::Standard => self.render_standard(token),
            crate::cli::terminal::DisplayProfile::Plain => self.render_plain(token),
        }
    }

    // ── Rich mode ─────────────────────────────────────────────────────────
    // Word-wrap aware, handles ANSI tokens (code blocks, bold markers).

    fn render_rich(&mut self, token: &str) {
        let mut stdout = io::stdout();
        let max_width = CAPS.width.saturating_sub(4) as usize;

        for ch in token.chars() {
            if ch == '\n' {
                let _ = write!(stdout, "\n");
                self.col = 0;
            } else {
                // Soft-wrap at terminal boundary
                if self.col >= max_width {
                    let _ = write!(stdout, "\n");
                    self.col = 0;
                }
                let _ = write!(stdout, "{}", ch);
                self.col += unicode_width::UnicodeWidthChar::width(ch).unwrap_or(1);
            }
        }
        let _ = stdout.flush();
    }

    // ── Standard mode ─────────────────────────────────────────────────────
    // Direct flush on every token, no word-wrap computation.

    fn render_standard(&mut self, token: &str) {
        let mut stdout = io::stdout();
        let _ = write!(stdout, "{}", token);
        let _ = stdout.flush();
    }

    // ── Plain mode ────────────────────────────────────────────────────────
    // Buffer whole lines and print them complete (no partial lines).

    fn render_plain(&mut self, token: &str) {
        let mut stdout = io::stdout();
        self.line_buf.push_str(token);
        while let Some(pos) = self.line_buf.find('\n') {
            let line = self.line_buf[..pos].to_string();
            self.line_buf = self.line_buf[pos + 1..].to_string();
            let _ = writeln!(stdout, "{}", line);
        }
        let _ = stdout.flush();
    }

    fn finish(&mut self) {
        let mut stdout = io::stdout();

        // Flush any remaining plain-mode buffer
        if !self.line_buf.is_empty() {
            let _ = writeln!(stdout, "{}", self.line_buf);
            self.line_buf.clear();
        }

        // Footer separator
        if CAPS.ansi {
            let _ = writeln!(stdout, "\n{}", Theme::separator(CAPS.width.saturating_sub(2) as usize));
        } else {
            let _ = writeln!(stdout, "\n{}", "-".repeat(60));
        }

        let _ = stdout.flush();
        self.col = 0;
        self.header_printed = false;
    }
}

// ─── Typewriter effect helper ─────────────────────────────────────────────────

/// Print a string with an optional typing animation (Rich mode only).
/// Falls back to instant print on Standard/Plain.
pub async fn typewriter(text: &str, delay_ms: u64) {
    if !CAPS.is_rich() || delay_ms == 0 {
        print!("{}", text);
        let _ = io::stdout().flush();
        return;
    }

    let mut stdout = io::stdout();
    for ch in text.chars() {
        let _ = write!(stdout, "{}", ch);
        let _ = stdout.flush();
        tokio::time::sleep(Duration::from_millis(delay_ms)).await;
    }
}

// ─── Non-streaming response printer ─────────────────────────────────────────

/// Print a complete response that was returned all at once.
/// Simulates streaming typewriter effect on Rich terminals so the UX
/// feels live even for non-streaming API calls.
pub async fn print_response(text: &str, simulate_streaming: bool) {
    let mut stdout = io::stdout();

    println!();

    if CAPS.ansi {
        let _ = writeln!(stdout, " {}", Theme::glyph_microdragon().to_string().with(Color::Cyan).bold());
        let _ = writeln!(stdout, "{}", Theme::separator(CAPS.width.saturating_sub(2) as usize));
    } else {
        let _ = writeln!(stdout, "\n[MICRODRAGON]");
        let _ = writeln!(stdout, "{}", "-".repeat(60));
    }

    if simulate_streaming && CAPS.is_rich() {
        // Fake streaming: chunk by word
        let words: Vec<&str> = text.split_inclusive(' ').collect();
        for word in words {
            print!("{}", word);
            let _ = io::stdout().flush();
            tokio::time::sleep(Duration::from_millis(8)).await;
        }
        println!();
    } else {
        println!("{}", text);
    }

    if CAPS.ansi {
        let _ = writeln!(stdout, "{}", Theme::separator(CAPS.width.saturating_sub(2) as usize));
    } else {
        let _ = writeln!(stdout, "{}", "-".repeat(60));
    }

    let _ = stdout.flush();
}
