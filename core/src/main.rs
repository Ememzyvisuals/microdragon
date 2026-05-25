// microdragon-core/src/main.rs
// © 2026 EMEMZYVISUALS DIGITALS — Emmanuel Ariyo

mod cli;
mod events;
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
mod tools;

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
    { let _ = crossterm::execute!(std::io::stdout(), crossterm::style::ResetColor); }

    // 3. Logging
    init_logging();

    // 4. Config
    let config = match MicrodragonConfig::load() {
        Ok(c)  => { info!("Config loaded"); c }
        Err(e) => {
            eprintln!("Config error: {}. Run 'microdragon setup'.", e);
            process::exit(1);
        }
    };

    // 5. Engine (tokio-rusqlite = fully Send+Sync, safe for tokio::spawn)
    let engine = match MicrodragonEngine::new(config.clone()).await {
        Ok(e)  => { info!("Engine ready"); Arc::new(e) }
        Err(e) => {
            error!("Engine init: {}", e);
            eprintln!("Engine error: {}", e);
            process::exit(1);
        }
    };

    // 6. API server background task (tokio::spawn is now safe — engine is Send+Sync)
    if config.is_configured() {
        let api_engine = engine.clone();
        let api_port = std::env::var("MICRODRAGON_API_PORT")
            .unwrap_or_else(|_| "7700".to_string())
            .parse::<u16>().unwrap_or(7700);
        tokio::spawn(async move {
            if let Err(e) = api::start_api_server(api_engine, api_port).await {
                error!("API server: {}", e);
            }
        });
        info!("API server on port {}", api_port);
    }

    // 7. Watch daemon
    if config.is_configured() {
        let watch_engine = engine.clone();
        tokio::spawn(async move {
            watch::start_watch_daemon(watch_engine).await;
        });
    }

    // 8. First launch
    if cli::first_launch::is_first_launch() {
        match cli::first_launch::run_first_launch().await {
            Ok(false) => process::exit(0),
            Err(e)    => eprintln!("First launch error: {}", e),
            Ok(true)  => {}
        }
    }

    // 9. TUI / Simple / CLI routing
    let args: Vec<String> = std::env::args().collect();
    let is_simple  = args.iter().any(|a| a == "--simple" || a == "-s");
    let is_cli_cmd = args.len() > 1 && !args[1].starts_with('-');

    if is_cli_cmd {
        // Subcommand mode: microdragon ask / code / research / etc.
        let cli = MicrodragonCli::new(engine);
        if let Err(e) = cli.run().await {
            error!("CLI error: {}", e);
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    } else if is_simple {
        // Fallback plain mode for terminals that don't support ratatui
        let mut simple = cli::simple_mode::SimpleMode::new(engine);
        if let Err(e) = simple.run().await {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    } else {
        // Default: full TUI (like OpenCode / Claude Code)
        if let Err(e) = cli::tui::run(engine).await {
            eprintln!("TUI error: {}", e);
            eprintln!("If your terminal doesn't support the TUI, run: microdragon --simple");
            process::exit(1);
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
    let level = std::env::var("MICRODRAGON_LOG")
        .unwrap_or_else(|_| "microdragon=info,warn".to_string());
    tracing_subscriber::fmt()
        .with_env_filter(level)
        .with_writer(non_blocking)
        .with_ansi(false)
        .init();
}
