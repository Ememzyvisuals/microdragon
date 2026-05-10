// microdragon-core/src/cli/display.rs
// Display helpers — graceful degradation on all terminal types.

use std::io;
use crossterm::style::{Color, Stylize};
use crate::cli::terminal::CAPS;
use crate::cli::animation::Spinner as AnimSpinner;

// ─── Spinner wrapper ──────────────────────────────────────────────────────────

pub struct Spinner(AnimSpinner);

impl Spinner {
    pub fn new(msg: &str) -> Self {
        Self(AnimSpinner::new(msg))
    }
    pub fn finish(mut self) {
        self.0.finish("done");
    }
    pub fn succeed(mut self, msg: &str) {
        self.0.finish_success(msg);
    }
    pub fn fail(mut self, msg: &str) {
        self.0.finish_error(msg);
    }
    pub fn set_message(&self, msg: impl Into<String>) {
        self.0.set_message(msg.into());
    }
}

// ─── Status prints ────────────────────────────────────────────────────────────

pub fn print_ok(msg: &str) {
    if CAPS.ansi_color {
        println!("  {} {}", "✓".with(Color::Rgb { r:0,g:255,b:136 }), msg);
    } else {
        println!("  [ok] {}", msg);
    }
}

pub fn print_err(msg: &str) {
    if CAPS.ansi_color {
        eprintln!("  {} {}", "✗".with(Color::Rgb { r:255,g:68,b:68 }), 
                  msg.to_string().with(Color::Rgb { r:255,g:68,b:68 }));
    } else {
        eprintln!("  [error] {}", msg);
    }
}

pub fn print_warn(msg: &str) {
    if CAPS.ansi_color {
        println!("  {} {}", "!".with(Color::Rgb { r:255,g:140,b:0 }),
                 msg.to_string().with(Color::Rgb { r:255,g:140,b:0 }));
    } else {
        println!("  [warn] {}", msg);
    }
}

pub fn print_info(msg: &str) {
    if CAPS.ansi_color {
        println!("  {} {}", "→".with(Color::DarkGrey), msg);
    } else {
        println!("  [info] {}", msg);
    }
}

pub fn response_footer(provider: &str, model: &str, tokens: u32, latency_ms: u64) {
    if CAPS.ansi_color {
        println!("\n  {} {} · {} · {}ms · {} tokens",
            "🐉".to_string().with(Color::Rgb { r:0,g:255,b:136 }),
            "microdragon".to_string().with(Color::Rgb { r:0,g:255,b:136 }),
            provider.to_string().with(Color::DarkGrey),
            latency_ms.to_string().with(Color::DarkGrey),
            tokens.to_string().with(Color::DarkGrey),
        );
    } else {
        println!("\n  [microdragon | {} | {}ms | {} tokens]", provider, latency_ms, tokens);
    }
    let _ = model;
}

pub fn print_divider() {
    if CAPS.ansi_color {
        println!("  {}", "─".repeat(60).with(Color::DarkGrey));
    } else {
        println!("  {}", "-".repeat(60));
    }
}

pub fn print_header(title: &str) {
    if CAPS.ansi_color {
        println!("\n  {}", title.to_string().with(Color::Rgb { r:0,g:255,b:136 }).bold());
        print_divider();
    } else {
        println!("\n  {}", title);
        println!("  {}", "-".repeat(title.len()));
    }
}

pub fn section(title: &str) {
    if crate::cli::terminal::CAPS.ansi_color {
        use crossterm::style::Stylize;
        println!("\n  {}", title.to_string().bold());
        println!("  {}", "─".repeat(50));
    } else {
        println!("\n  {}", title);
        println!("  {}", "-".repeat(50));
    }
}

pub fn kv(key: &str, value: &str) {
    println!("  {:20} {}", key, value);
}

pub fn kv_colored(key: &str, value: &str, color: crossterm::style::Color) {
    if crate::cli::terminal::CAPS.ansi_color {
        use crossterm::style::Stylize;
        println!("  {:20} {}", key, value.to_string().with(color));
    } else {
        println!("  {:20} {}", key, value);
    }
}

pub fn panel(title: &str, body: &str) {
    let width = 60usize;
    if crate::cli::terminal::CAPS.ansi_color {
        use crossterm::style::Stylize;
        println!("\n  ┌{}┐", "─".repeat(width));
        println!("  │ {:<width$}│", title.bold().to_string(), width=width-1);
        println!("  │{}│", " ".repeat(width));
        for line in body.lines() {
            let chunks = line.chars().collect::<Vec<_>>();
            let chunks = chunks.chunks(width - 2);
            for chunk in chunks {
                let s: String = chunk.iter().collect();
                println!("  │ {:<width$}│", s, width=width-1);
            }
        }
        println!("  └{}┘", "─".repeat(width));
    } else {
        println!("\n  {}", title);
        println!("  {}", "-".repeat(width));
        println!("  {}", body);
        println!();
    }
}
