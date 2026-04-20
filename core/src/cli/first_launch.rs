// microdragon/core/src/cli/first_launch.rs
//
// Pixel-perfect OpenClaw-style CLI launch experience.
// Studied from reference image: dark terminal, dense block art, 2-color scheme.
//
// EXACT structure from OpenClaw image:
//  Line 1:  🦞 OpenClaw 2026.3.2 (85377a2)         ← colored version line
//  Line 2:  Give me a workspace and I'll...          ← green tagline
//  (blank)
//  ┌──────────────────────────────────────────────┐
//  │ ████ ████ █████ █  █  ███  █   █ █████ ███  │  ← dense pixel-block name
//  └──────────────────────────────────────────────┘
//           ▼ OPENCLAW ▼                            ← centered identity label
//
//  OpenClaw onboarding                              ← green onboarding header
//  Security ─────────────────────────────          ← red section header
//  ┌────────────────────────────────────────────┐
//  │ Security warning — please read.            │  ← bordered warning
//  │ ...                                        │
//  └────────────────────────────────────────────┘
//  ● I understand...                            ← orange bullet confirmations
//  ● Yes / ○ No                                ← green yes, dim no
//
// © 2026 EMEMZYVISUALS DIGITALS — Emmanuel Ariyo

use anyhow::Result;
use std::io::{self, Write};
use std::path::PathBuf;
use crossterm::style::{Color, Stylize};
use crate::cli::theme::{Theme, random_tagline};

const VERSION: &str = env!("CARGO_PKG_VERSION");

// ── Pixel block art — MICRODRAGON in dense LCD/dot-matrix style ──────────────
// Each glyph is crafted to be dense and readable at 80-column width.
// Uses full-block (█) and partial blocks (▀▄) for texture like OpenClaw.
// Width: ~76 chars per row, fits in 80-col terminal with 2-char left margin.

const BANNER: [&str; 5] = [
    "█▀▄▀█ █ █▀▀ █▀█ █▀█ █▀▄ █▀█ ▄▀█ █▀▀ █▀█ █▄░█",
    "█░▀░█ █ █▄▄ █▀▄ █▄█ █▄▀ █▀▄ █▀█ █▄█ █▄█ █░▀█",
    "─────────────────────────────────────────────",
    "        🐉  D R A G O N   E N G I N E        ",
    "            I G N I T E  v0.1.1              ",
];

fn marker_path() -> std::path::PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("microdragon")
        .join(".accepted_terms")
}

pub fn is_first_launch() -> bool { !marker_path().exists() }

pub fn mark_launched() -> Result<()> {
    let p = marker_path();
    if let Some(d) = p.parent() { std::fs::create_dir_all(d)?; }
    std::fs::write(&p, VERSION)?;
    Ok(())
}

pub async fn run_first_launch() -> Result<bool> {
    print_launch();

    let ok1 = confirm("I understand this is personal-by-default and shared/multi-user use requires lock-down.")?;
    if !ok1 { declined(); return Ok(false); }
    let ok2 = confirm("I have read the security notice and accept responsibility for how I use Microdragon.")?;
    if !ok2 { declined(); return Ok(false); }

    mark_launched()?;
    println!();
    println!("  {}  Run {} to configure your AI provider.",
        "🐉".to_string().with(Theme::GREEN),
        "microdragon setup".to_string().with(Theme::GREEN).bold()
    );
    println!();
    Ok(true)
}

fn print_launch() {
    let tagline = random_tagline();
    let hash    = option_env!("GIT_SHORT_SHA").unwrap_or("dev");
    println!();

    // ── Row 1: version + hash ── FIRE RED bold, matches OpenClaw exactly
    println!("  {} {}",
        "🐉".to_string().with(Theme::FIRE),
        format!("Microdragon {} ({})  — Ignite Engine", VERSION, hash)
            .with(Theme::FIRE).bold()
    );

    // ── Row 2: tagline ── GREEN, changes every launch
    println!("  {}", tagline.to_string().with(Theme::GREEN));
    println!();

    // ── Banner box ── pixel block art inside a clean border
    let bw: usize = 50;
    println!("  ╔{}╗", "═".repeat(bw));
    println!("  ║{}║", " ".repeat(bw));
    for (i, row) in BANNER.iter().enumerate() {
        let rlen = row.chars().count();
        let lpad = (bw.saturating_sub(rlen)) / 2;
        let rpad = bw.saturating_sub(rlen + lpad);
        if i == 2 {
            // Separator line — dim
            println!("  ║{}║",
                format!("{}{}{}", " ".repeat(lpad), row, " ".repeat(rpad))
                    .with(Color::DarkGrey));
        } else if i == 3 || i == 4 {
            // Dragon label rows — ember/green alternating
            let color = if i == 3 { Theme::GREEN } else { Theme::EMBER };
            println!("  ║{}║",
                format!("{}{}{}", " ".repeat(lpad), row, " ".repeat(rpad))
                    .with(color).bold());
        } else {
            // Main pixel art — bright green
            println!("  ║{}║",
                format!("{}{}{}", " ".repeat(lpad), row, " ".repeat(rpad))
                    .with(Theme::GREEN).bold());
        }
    }
    println!("  ║{}║", " ".repeat(bw));
    println!("  ╚{}╝", "═".repeat(bw));

    // Identity label — mirrors "▼ OPENCLAW ▼"
    let label = "🐉  MICRODRAGON  🐉";
    let total_w = bw + 4; // account for "  ╔" + "╗"
    let lvis  = label.chars().count() + 4; // emoji width
    let lpad  = total_w.saturating_sub(lvis) / 2;
    println!("  {}{}", " ".repeat(lpad),
        label.to_string().with(Theme::GREEN).bold());
    println!();

    // ── Onboarding header ── green bold, mirrors "OpenClaw onboarding"
    println!("  {}", "Microdragon onboarding".to_string().with(Theme::GREEN).bold());
    println!();

    // ── Security section ── red bold + dim dashes
    let sec_dash = "─".repeat(bw.saturating_sub("Security ".len()));
    println!("  {} {}",
        "Security".to_string().with(Theme::FIRE).bold(),
        sec_dash.to_string().with(Color::DarkGrey)
    );
    println!();

    // ── Security box ── bordered, matches OpenClaw's warning block precisely
    let iw = bw + 4;
    let security_text = [
        "Security warning — please read.",
        "",
        "Microdragon is a power tool in public beta. Expect sharp edges.",
        "By default, it's a personal agent: one trusted operator boundary.",
        "This agent can read files and run actions if tools are enabled.",
        "A crafted prompt could trick it into unsafe behaviour.",
        "",
        "Microdragon is NOT designed for multi-tenant deployment.",
        "Multiple users on one instance share full tool authority.",
        "",
        "Cybersecurity: ALL offensive features require WRITTEN authorisation.",
        "You are legally responsible for how you use this tool.",
        "",
        "Recommended baseline:",
        "  - One operator per instance",
        "  - Keep API keys out of prompts",
        "  - Sandbox execution is ON by default — keep it on",
        "  - Review: ~/.local/share/microdragon/audit.log",
        "",
        "Run:  microdragon security audit --deep",
        "Docs: https://github.com/ememzyvisuals/microdragon",
    ];
    println!("  ┌{}┐", "─".repeat(iw));
    for line in &security_text {
        if line.is_empty() {
            println!("  │{}│", " ".repeat(iw));
        } else {
            let s = if line.len() > iw.saturating_sub(2) { &line[..iw.saturating_sub(2)] } else { line };
            let p = iw.saturating_sub(s.len() + 1);
            println!("  │ {}{}│", s, " ".repeat(p));
        }
    }
    println!("  └{}┘", "─".repeat(iw));
    println!();
}

fn confirm(stmt: &str) -> Result<bool> {
    // Orange bullet + white text — matches OpenClaw confirmation lines exactly
    println!("  {} {}",
        "●".to_string().with(Theme::EMBER),
        stmt.to_string().with(Color::White)
    );
    print!("  {}   {}",
        "● Yes".to_string().with(Theme::GREEN).bold(),
        "/ ○ No".to_string().with(Color::DarkGrey)
    );
    io::stdout().flush()?;
    let mut buf = String::new();
    io::stdin().read_line(&mut buf)?;
    Ok(matches!(buf.trim().to_lowercase().as_str(), "y" | "yes"))
}

fn declined() {
    println!();
    println!("  {} Run {} again when ready.",
        "Declined.".to_string().with(Theme::FIRE),
        "microdragon".to_string().with(Theme::GREEN)
    );
    println!();
}
