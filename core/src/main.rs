// microdragon-core/src/main.rs
// MICRODRAGON Universal AI Agent — Core Engine Entry Point
// © 2026 EMEMZYVISUALS DIGITALS — Founder & CEO: Emmanuel Ariyo
// https://github.com/ememzyvisuals/microdragon

mod cli;
mod engine;
mod brain;
mod config;
mod memory;
mod security;
mod ipc;
mod api;
mod watch;
mod skills;
mod harness;
mod mcp;


use anyhow::Result;
use tracing::{info, error};
use std::sync::Arc;
use std::process;

use crate::cli::MicrodragonCli;
use crate::engine::MicrodragonEngine;
use crate::config::MicrodragonConfig;
use crate::cli::terminal::CAPS;

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Terminal detection
    let _ = &*CAPS;

    // 2. Windows ANSI enable
    #[cfg(target_os = "windows")]
    {
        let _ = crossterm::execute!(std::io::stdout(), crossterm::style::ResetColor);
    }

    // 3. Logging
    init_logging();

    // 4. Configuration
    let config = match MicrodragonConfig::load() {
        Ok(c) => { info!("Config loaded"); c }
        Err(e) => {
            use crate::cli::theme::Theme;
            eprintln!("{} Config error: {}", Theme::glyph_err(), e);
            eprintln!("Run 'microdragon setup' to configure MICRODRAGON.");
            process::exit(1);
        }
    };

    // 5. Engine
    let engine = match MicrodragonEngine::new(config.clone()).await {
        Ok(e) => { info!("MICRODRAGON Engine ready"); Arc::new(e) }
        Err(e) => {
            use crate::cli::theme::Theme;
            error!("Engine init failed: {}", e);
            eprintln!("{} Engine error: {}", Theme::glyph_err(), e);
            process::exit(1);
        }
    };

    // 6. API server (background — social bots connect here)
    if config.is_configured() {
        let api_engine = engine.clone();
        let api_port = std::env::var("MICRODRAGON_API_PORT")
            .unwrap_or_else(|_| "7700".to_string())
            .parse::<u16>().unwrap_or(7700);
        tokio::spawn(async move {
            if let Err(e) = api::start_api_server(api_engine, api_port).await {
                error!("API server error: {}", e);
            }
        });
        info!("API server starting on port {}", 7700);
    }

    // 7. Watch daemon (background)
    if config.is_configured() {
        let watch_engine = engine.clone();
        tokio::spawn(async move {
            watch::start_watch_daemon(watch_engine).await;
        });
    }

    // 8. First-launch check
    {
        if cli::first_launch::is_first_launch() {
            match cli::first_launch::run_first_launch().await {
                Ok(false) => {
                    // User declined terms
                    std::process::exit(0);
                }
                Err(e) => {
                    eprintln!("First launch error: {}", e);
                }
                Ok(true) => {
                    // Accepted — proceed to setup
                }
            }
        }
    }

    // 9. Check for Pro Mode flag, otherwise start Simple Mode
    let args: Vec<String> = std::env::args().collect();
    let is_pro = args.iter().any(|a| a == "--pro" || a == "-p");

    if is_pro || args.len() > 1 {
        // Pro Mode — full CLI
        let cli = MicrodragonCli::new(engine);
        if let Err(e) = cli.run().await {
        use crate::cli::theme::Theme;
        error!("CLI error: {}", e);
        eprintln!("{} {}", Theme::error_str(Theme::glyph_err()), e);
        process::exit(1);
        }
    } else {
        // Simple Mode — conversational default
        let mut simple = cli::simple_mode::SimpleMode::new(engine);
        if let Err(e) = simple.run().await {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
        if simple.is_pro_mode() {
            // User typed /pro — restart in pro mode
            eprintln!("Restart with: microdragon --pro");
        }
    }

    Ok(())
}

fn init_logging() {
    let log_dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("microdragon").join("logs");
    std::fs::create_dir_all(&log_dir).ok();

    let file_appender = tracing_appender::rolling::daily(&log_dir, "microdragon.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    let env_filter = std::env::var("MICRODRAGON_LOG").unwrap_or_else(|_| "microdragon=info,warn".to_string());
    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_writer(non_blocking)
        .with_ansi(false)
        .init();
}
