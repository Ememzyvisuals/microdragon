// microdragon-core/src/cli/tui/mod.rs
//
// MICRODRAGON — Full Terminal UI
//
// Layout:
//   ┌─────────────────────────────────────────────────────┐
//   │  # Task title                       tokens  cost   │  ← header
//   ├─────────────────────────────────────────────────────┤
//   │                                                     │
//   │   agent output scrolls here                         │  ← log area
//   │   * Grep "pattern"                                  │
//   │   → Read src/main.rs                                │
//   │   ~ Thinking...                                     │
//   │                                                     │
//   ├─────────────────────────────────────────────────────┤
//   │  >  input bar                                       │  ← input
//   ├─────────────────────────────────────────────────────┤
//   │  provider · model    ctrl+p  /help  tab  esc        │  ← shortcuts
//   └─────────────────────────────────────────────────────┘
//
// © 2026 EMEMZYVISUALS DIGITALS — Emmanuel Ariyo

pub mod events;

use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture,
        Event, KeyCode, KeyModifiers,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};

use crate::engine::MicrodragonEngine;
use crate::events::{AgentEvent, EventKind, EventRx, EventTx};

// ─── ASCII Logo ────────────────────────────────────────────────────────────────

const LOGO: &[&str] = &[
    r"  ███╗   ███╗██╗ ██████╗██████╗  ██████╗ ██████╗  █████╗  ██████╗  ██████╗ ███╗  ██╗",
    r"  ████╗ ████║██║██╔════╝██╔══██╗██╔═══██╗██╔══██╗██╔══██╗██╔════╝ ██╔═══██╗████╗ ██║",
    r"  ██╔████╔██║██║██║     ██████╔╝██║   ██║██║  ██║███████║██║  ███╗██║   ██║██╔██╗██║",
    r"  ██║╚██╔╝██║██║██║     ██╔══██╗██║   ██║██║  ██║██╔══██║██║   ██║██║   ██║██║╚████║",
    r"  ██║ ╚═╝ ██║██║╚██████╗██║  ██║╚██████╔╝██████╔╝██║  ██║╚██████╔╝╚██████╔╝██║ ╚███║",
    r"  ╚═╝     ╚═╝╚═╝ ╚═════╝╚═╝  ╚═╝ ╚═════╝ ╚═════╝ ╚═╝  ╚═╝ ╚═════╝  ╚═════╝╚═╝  ╚══╝",
];

const LOGO_SUBTITLE: &str = "  Distributed Intelligence Network  ·  Cognitive Autonomous Agent  ·  EMEMZYVISUALS DIGITALS";

// ─── Colour palette ────────────────────────────────────────────────────────────

const C_BG:       Color = Color::Rgb(12, 12, 16);   // near-black
const C_SURFACE:  Color = Color::Rgb(20, 20, 28);   // panel bg
const C_BORDER:   Color = Color::Rgb(45, 45, 60);   // subtle border
const C_GREEN:    Color = Color::Rgb(0, 255, 136);   // brand green
const C_EMBER:    Color = Color::Rgb(255, 160, 60);   // brand ember
const C_FIRE:     Color = Color::Rgb(255, 68, 68);   // error red
const C_CYAN:     Color = Color::Rgb(80, 220, 255);   // tool calls
const C_BLUE:     Color = Color::Rgb(100, 160, 255);   // file reads
const C_YELLOW:   Color = Color::Rgb(255, 220, 80);   // web search
const C_PURPLE:   Color = Color::Rgb(180, 120, 255);   // memory
const C_DIM:      Color = Color::Rgb(80, 80, 100);   // dim text
const C_WHITE:    Color = Color::Rgb(210, 210, 220);   // body text
const C_TITLE:    Color = Color::Rgb(240, 240, 255);   // headings

// ─── Slash commands ────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
struct SlashCommand {
    name:        &'static str,
    description: &'static str,
}

const SLASH_COMMANDS: &[SlashCommand] = &[
    SlashCommand { name: "/help",     description: "Show all commands and shortcuts" },
    SlashCommand { name: "/setup",    description: "Configure API key and provider (run anytime)" },
    SlashCommand { name: "/key",      description: "Set API key instantly: /key gsk_... or /key sk-..." },
    SlashCommand { name: "/clear",    description: "Clear the conversation log" },
    SlashCommand { name: "/status",   description: "Show engine and provider status" },
    SlashCommand { name: "/memory",   description: "Show recent conversation memory" },
    SlashCommand { name: "/models",   description: "List available AI models" },
    SlashCommand { name: "/provider", description: "Show or switch AI provider" },
    SlashCommand { name: "/agents",   description: "List all agent capabilities" },
    SlashCommand { name: "/research", description: "Run a research task with web search" },
    SlashCommand { name: "/code",     description: "Enter code generation mode" },
    SlashCommand { name: "/exit",     description: "Exit Microdragon" },
];

// ─── App state ─────────────────────────────────────────────────────────────────

#[derive(Debug, PartialEq)]
enum AppMode {
    Normal,
    /// Showing slash command palette
    CommandPalette,
    /// Waiting for pipeline response
    Working,
}

struct App {
    engine:          Arc<MicrodragonEngine>,
    mode:            AppMode,

    /// Input bar content
    input:           String,
    input_cursor:    usize,

    /// Full agent event log
    log:             Vec<AgentEvent>,
    log_scroll:      usize,
    log_list_state:  ListState,

    /// Current task info
    task_title:      Option<String>,
    total_tokens:    u32,
    total_cost_usd:  f64,
    start_time:      Option<Instant>,

    /// Session info
    provider:        String,
    model:           String,
    session_turns:   u32,

    /// Slash command palette
    palette_filter:  String,
    palette_index:   usize,

    /// Event channel from pipeline
    event_rx:        EventRx,

    /// Channel for sending events INTO the TUI from background tasks
    event_tx:        EventTx,

    /// Thinking animation frame
    thinking_frame:  usize,

    /// Quit flag
    should_quit:     bool,
}

const THINKING_FRAMES: &[&str] = &[
    "·       ", "··      ", "···     ", "····    ",
    "·····   ", "······  ", "······· ", "········",
    "········", "······· ", "······  ", "·····   ",
    "····    ", "···     ", "··      ", "·       ",
];

impl App {
    async fn new(engine: Arc<MicrodragonEngine>) -> Self {
        let config = engine.get_config().await;
        let provider = config.ai.active_provider.to_string();
        let model = config.ai.providers.get_model(&config.ai.active_provider);

        let (tx, rx) = events::channel();

        Self {
            engine,
            mode: AppMode::Normal,
            input: String::new(),
            input_cursor: 0,
            log: Vec::new(),
            log_scroll: 0,
            log_list_state: ListState::default(),
            task_title: None,
            total_tokens: 0,
            total_cost_usd: 0.0,
            start_time: None,
            provider,
            model,
            session_turns: 0,
            palette_filter: String::new(),
            palette_index: 0,
            event_rx: rx,
            event_tx: tx,
            thinking_frame: 0,
            should_quit: false,
        }
    }

    fn push_event(&mut self, event: AgentEvent) {
        self.log.push(event);
        // Auto-scroll to bottom
        self.log_scroll = self.log.len().saturating_sub(1);
    }

    fn push(&mut self, kind: EventKind, text: impl Into<String>) {
        self.push_event(AgentEvent::new(kind, text));
    }

    fn visible_commands(&self) -> Vec<&SlashCommand> {
        if self.palette_filter.is_empty() {
            SLASH_COMMANDS.iter().collect()
        } else {
            SLASH_COMMANDS.iter()
                .filter(|c| c.name.contains(&self.palette_filter)
                         || c.description.to_lowercase().contains(&self.palette_filter.to_lowercase()))
                .collect()
        }
    }

    fn cost_str(&self) -> String {
        if self.total_cost_usd < 0.001 {
            String::new()
        } else {
            format!("(${:.2})", self.total_cost_usd)
        }
    }

    /// Estimate cost based on tokens (Groq free / OpenAI pricing)
    fn estimate_cost(&self) -> f64 {
        // Approximate: $0.59 per 1M tokens (Groq llama3)
        self.total_tokens as f64 * 0.00000059
    }
}

// ─── Main TUI entry point ─────────────────────────────────────────────────────

pub async fn run(engine: Arc<MicrodragonEngine>) -> Result<()> {
    // ── Inline setup detection ────────────────────────────────────────────────
    // If not configured, run the setup wizard right here before entering the TUI.
    // User never has to exit and run a separate command.
    let config = engine.get_config().await;
    if !config.is_configured() {
        println!();
        println!("  🐉  Welcome to MICRODRAGON");
        println!("  No API key configured. Let's fix that right now.");
        println!("  ─────────────────────────────────────────────────");
        println!();

        let wizard = crate::cli::setup::SetupWizard::new(Arc::clone(&engine));
        wizard.run().await?;

        // Re-check after setup
        let config = engine.get_config().await;
        if !config.is_configured() {
            println!();
            println!("  Setup skipped. Run 'microdragon setup' to configure.");
            println!("  Launching in read-only mode…");
            println!();
        } else {
            println!();
            println!("  ✓ Configuration saved. Launching MICRODRAGON…");
            println!();
            // small pause so user sees the success message
            tokio::time::sleep(tokio::time::Duration::from_millis(800)).await;
        }
    }

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run app
    let result = run_app(&mut terminal, engine).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    engine: Arc<MicrodragonEngine>,
) -> Result<()> {
    let mut app = App::new(engine).await;

    // Welcome — show full ASCII logo
    for line in LOGO {
        app.push(EventKind::Thought, *line);
    }
    app.push(EventKind::Thought, LOGO_SUBTITLE);
    app.push(EventKind::Divider, "─".repeat(88));

    let config = app.engine.get_config().await;
    if config.is_configured() {
        app.push(EventKind::Done, format!(
            "Provider: {}  ·  Model: {}  ·  9-phase agentic pipeline ready",
            app.provider, app.model
        ));
        app.push(EventKind::Thought, "  Type your task below, or / for commands.");
    } else {
        app.push(EventKind::Warning,
            "Not configured — no API key set yet.");
        app.push(EventKind::Thought,
            "  Type  /key gsk_xxxx       to add a Groq key  (free at console.groq.com)");
        app.push(EventKind::Thought,
            "  Type  /key sk-ant-xxxx    to add an Anthropic key");
        app.push(EventKind::Thought,
            "  Type  /key sk-xxxx        to add an OpenAI key");
        app.push(EventKind::Thought,
            "  Type  /setup              to run the full setup wizard");
    }
    app.push(EventKind::Divider, "─".repeat(88));

    let tick_rate = Duration::from_millis(80);
    let mut last_tick = Instant::now();

    loop {
        // Drain any pending agent events from background tasks
        loop {
            match app.event_rx.try_recv() {
                Ok(ev) => {
                    if matches!(ev.kind, EventKind::Done) {
                        app.mode = AppMode::Normal;
                    }
                    if let EventKind::Response = ev.kind {
                        // Count tokens approximation from response length
                        app.total_tokens += (ev.text.len() / 4) as u32;
                        app.total_cost_usd = app.estimate_cost();
                    }
                    app.push_event(ev);
                }
                Err(_) => break,
            }
        }

        // Draw
        terminal.draw(|f| draw(f, &mut app))?;

        // Poll for keyboard events
        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                handle_key(&mut app, key).await;
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.thinking_frame = (app.thinking_frame + 1) % THINKING_FRAMES.len();
            last_tick = Instant::now();
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

// ─── Key handling ─────────────────────────────────────────────────────────────

async fn handle_key(app: &mut App, key: crossterm::event::KeyEvent) {
    match app.mode {
        AppMode::CommandPalette => handle_key_palette(app, key).await,
        AppMode::Working        => handle_key_working(app, key),
        AppMode::Normal         => handle_key_normal(app, key).await,
    }
}

fn handle_key_working(app: &mut App, key: crossterm::event::KeyEvent) {
    // Only allow Escape to interrupt while working
    if key.code == KeyCode::Esc ||
       (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL)) {
        app.push(EventKind::Warning, "Interrupted by user");
        app.mode = AppMode::Normal;
    }
}

async fn handle_key_palette(app: &mut App, key: crossterm::event::KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.mode = AppMode::Normal;
            app.input = String::new();
            app.palette_filter = String::new();
        }
        KeyCode::Enter => {
            let cmds = app.visible_commands();
            if let Some(cmd) = cmds.get(app.palette_index) {
                let cmd_name = cmd.name.to_string();
                app.input = String::new();
                app.palette_filter = String::new();
                app.mode = AppMode::Normal;
                execute_slash_command(app, &cmd_name).await;
            }
        }
        KeyCode::Up => {
            if app.palette_index > 0 { app.palette_index -= 1; }
        }
        KeyCode::Down => {
            let max = app.visible_commands().len().saturating_sub(1);
            if app.palette_index < max { app.palette_index += 1; }
        }
        KeyCode::Tab => {
            let max = app.visible_commands().len().saturating_sub(1);
            app.palette_index = (app.palette_index + 1).min(max);
        }
        KeyCode::Char(c) => {
            app.palette_filter.push(c);
            app.palette_index = 0;
        }
        KeyCode::Backspace => {
            app.palette_filter.pop();
        }
        _ => {}
    }
}

async fn handle_key_normal(app: &mut App, key: crossterm::event::KeyEvent) {
    match key.code {
        // Submit input
        KeyCode::Enter => {
            let input = app.input.trim().to_string();
            if input.is_empty() { return; }

            app.input.clear();
            app.input_cursor = 0;

            if input.starts_with('/') {
                execute_slash_command(app, &input).await;
            } else {
                submit_task(app, input).await;
            }
        }

        // Type characters
        KeyCode::Char('/') if app.input.is_empty() => {
            app.input.push('/');
            app.input_cursor = 1;
            app.mode = AppMode::CommandPalette;
            app.palette_filter = String::new();
            app.palette_index = 0;
        }

        KeyCode::Char(c) => {
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                match c {
                    'c' | 'q' => app.should_quit = true,
                    'l' => { app.log.clear(); app.log_scroll = 0; }
                    'p' => {
                        app.mode = AppMode::CommandPalette;
                        app.palette_filter = String::new();
                        app.palette_index = 0;
                    }
                    _ => {}
                }
            } else {
                app.input.insert(app.input_cursor, c);
                app.input_cursor += 1;
            }
        }

        KeyCode::Backspace => {
            if app.input_cursor > 0 {
                app.input_cursor -= 1;
                app.input.remove(app.input_cursor);
            }
        }
        KeyCode::Delete => {
            if app.input_cursor < app.input.len() {
                app.input.remove(app.input_cursor);
            }
        }
        KeyCode::Left  => { if app.input_cursor > 0 { app.input_cursor -= 1; } }
        KeyCode::Right => { if app.input_cursor < app.input.len() { app.input_cursor += 1; } }
        KeyCode::Home  => { app.input_cursor = 0; }
        KeyCode::End   => { app.input_cursor = app.input.len(); }

        // Scroll log
        KeyCode::PageUp => {
            app.log_scroll = app.log_scroll.saturating_sub(10);
        }
        KeyCode::PageDown => {
            let max = app.log.len().saturating_sub(1);
            app.log_scroll = (app.log_scroll + 10).min(max);
        }
        KeyCode::Up => {
            app.log_scroll = app.log_scroll.saturating_sub(1);
        }
        KeyCode::Down => {
            let max = app.log.len().saturating_sub(1);
            app.log_scroll = (app.log_scroll + 1).min(max);
        }

        KeyCode::Esc => {
            app.input.clear();
            app.input_cursor = 0;
        }
        KeyCode::Tab => {
            // Show command palette
            app.mode = AppMode::CommandPalette;
            app.palette_filter = String::new();
            app.palette_index = 0;
        }
        _ => {}
    }
}

// ─── Submit task ──────────────────────────────────────────────────────────────

async fn submit_task(app: &mut App, input: String) {
    app.session_turns += 1;
    app.task_title = Some(input.chars().take(60).collect());
    app.start_time = Some(Instant::now());
    app.mode = AppMode::Working;

    app.push(EventKind::Divider, "─".repeat(60));
    app.push(EventKind::UserInput, format!("▸ {}", input));
    app.push(EventKind::Divider, "─".repeat(60));

    let engine = Arc::clone(&app.engine);
    let tx = app.event_tx.clone();
    let input_clone = input.clone();

    // Run pipeline in background, emit events to TUI
    tokio::spawn(async move {
        let _ = tx.send(AgentEvent::new(EventKind::PhaseStart, "1/9 PERCEIVE — Classifying intent…"));

        let progress_tx = tx.clone();
        let progress_cb = move |phase: u8, name: &'static str, detail: &str| {
            let kind = if phase <= 3 {
                EventKind::ToolCall
            } else {
                EventKind::PhaseStart
            };
            let _ = progress_tx.send(AgentEvent::new(
                kind,
                format!("{}/9 {}  {}", phase, name, detail),
            ));
        };

        match engine.process_command_with_progress(&input_clone, &progress_cb).await {
            Ok(result) => {
                let _ = tx.send(AgentEvent::new(EventKind::Divider, "─".repeat(60)));
                // Split response into lines for clean display
                for line in result.response.lines() {
                    let _ = tx.send(AgentEvent::new(EventKind::Response, line.to_string()));
                }
                let _ = tx.send(AgentEvent::new(EventKind::Divider, "─".repeat(60)));
                let _ = tx.send(AgentEvent::new(
                    EventKind::Done,
                    format!(
                        "{} · {} · {}ms · {} tokens",
                        result.provider, result.model,
                        result.latency_ms, result.tokens_used
                    ),
                ));
            }
            Err(e) => {
                let _ = tx.send(AgentEvent::new(EventKind::Error, format!("Pipeline error: {}", e)));
                let _ = tx.send(AgentEvent::new(EventKind::Done, "failed".to_string()));
            }
        }
    });
}

// ─── Slash commands ───────────────────────────────────────────────────────────

async fn execute_slash_command(app: &mut App, cmd: &str) {
    let cmd_lower = cmd.to_lowercase();
    let cmd_lower = cmd_lower.trim();

    match cmd_lower {
        "/exit" | "/quit" => {
            app.should_quit = true;
        }

        "/setup" => {
            // Exit TUI, run setup, then re-enter TUI
            app.should_quit = true;
            app.push(EventKind::Thought, "Exiting to run setup wizard…");
            app.push(EventKind::Thought, "After setup completes, run 'microdragon' to relaunch.");
            // Mark that we should run setup on exit
            // We do this by pushing a special sentinel — the main() handles it
            app.push(EventKind::Done, "RUN_SETUP_ON_EXIT");
        }

        cmd if cmd.starts_with("/key ") => {
            let raw_key = cmd.trim_start_matches("/key ").trim().to_string();
            if raw_key.is_empty() {
                app.push(EventKind::Warning, "Usage: /key <your_api_key>");
                app.push(EventKind::Thought, "Example: /key gsk_xxxx  or  /key sk-ant-xxxx");
            } else {
                let engine = Arc::clone(&app.engine);
                let tx = app.event_tx.clone();
                tokio::spawn(async move {
                    let mut config = engine.get_config().await;
                    let provider_name = if raw_key.starts_with("gsk_") {
                        config.ai.active_provider = crate::config::providers::ModelProvider::Groq;
                        config.ai.providers.groq_api_key = Some(raw_key.clone());
                        "Groq"
                    } else if raw_key.starts_with("sk-ant-") {
                        config.ai.active_provider = crate::config::providers::ModelProvider::Anthropic;
                        config.ai.providers.anthropic_api_key = Some(raw_key.clone());
                        "Anthropic"
                    } else if raw_key.starts_with("sk-or-") {
                        config.ai.active_provider = crate::config::providers::ModelProvider::OpenRouter;
                        config.ai.providers.openrouter_api_key = Some(raw_key.clone());
                        "OpenRouter"
                    } else if raw_key.starts_with("sk-") {
                        config.ai.active_provider = crate::config::providers::ModelProvider::OpenAI;
                        config.ai.providers.openai_api_key = Some(raw_key.clone());
                        "OpenAI"
                    } else {
                        config.ai.providers.groq_api_key = Some(raw_key.clone());
                        config.ai.active_provider = crate::config::providers::ModelProvider::Groq;
                        "Groq (assumed)"
                    };
                    match engine.update_config(config).await {
                        Ok(_) => {
                            let _ = tx.send(AgentEvent::new(
                                EventKind::Done,
                                format!("✓ {} key saved — ready immediately. No restart needed.", provider_name)
                            ));
                        }
                        Err(e) => {
                            let _ = tx.send(AgentEvent::new(
                                EventKind::Error,
                                format!("Failed to save key: {}", e)
                            ));
                        }
                    }
                });
            }
        }

        "/clear" => {
            app.log.clear();
            app.log_scroll = 0;
            app.task_title = None;
            app.total_tokens = 0;
            app.total_cost_usd = 0.0;
            app.push(EventKind::Thought, "Conversation cleared.");
        }

        "/help" => {
            app.push(EventKind::Divider, "─".repeat(60));
            app.push(EventKind::Thought, "🐉  MICRODRAGON — Commands & Shortcuts");
            app.push(EventKind::Divider, "─".repeat(60));
            for cmd in SLASH_COMMANDS {
                app.push(EventKind::ToolCall, format!("{:<12}  {}", cmd.name, cmd.description));
            }
            app.push(EventKind::Divider, "─".repeat(60));
            app.push(EventKind::Thought, "Shortcuts:");
            app.push(EventKind::Thought, "  ctrl+p  Open command palette");
            app.push(EventKind::Thought, "  tab     Browse commands");
            app.push(EventKind::Thought, "  esc     Cancel / clear input");
            app.push(EventKind::Thought, "  ctrl+c  Quit");
            app.push(EventKind::Thought, "  PgUp/PgDn  Scroll log");
            app.push(EventKind::Divider, "─".repeat(60));
        }

        "/status" => {
            let health = app.engine.health_check().await;
            app.push(EventKind::Divider, "─".repeat(60));
            app.push(EventKind::Thought, "🐉  MICRODRAGON Status");
            app.push(if health.is_healthy { EventKind::Done } else { EventKind::Warning },
                format!("Configured:  {}", if health.is_healthy { "yes ✓" } else { "NO — type /key <your_api_key> to fix now" }));
            app.push(EventKind::Done, format!("Provider:    {}", health.provider));
            app.push(EventKind::Done, format!("Model:       {}", health.model));
            app.push(EventKind::Done, format!("Memory:      {}", if health.memory_ok { "ok" } else { "error" }));
            app.push(EventKind::Done, format!("Turns:       {}", app.session_turns));
            app.push(EventKind::Done, format!("Tokens used: {}", app.total_tokens));
            if !app.cost_str().is_empty() {
                app.push(EventKind::Done, format!("Est. cost:   {}", app.cost_str()));
            }
            if !health.is_healthy {
                app.push(EventKind::Divider, "─".repeat(60));
                app.push(EventKind::Warning, "Quick fix — type one of these:");
                app.push(EventKind::Thought, "  /key gsk_xxxx              (Groq — free)");
                app.push(EventKind::Thought, "  /key sk-ant-xxxx           (Anthropic)");
                app.push(EventKind::Thought, "  /key sk-xxxx               (OpenAI)");
                app.push(EventKind::Thought, "  /setup                     (full wizard)");
            }
            app.push(EventKind::Divider, "─".repeat(60));
        }

        "/models" => {
            app.push(EventKind::Divider, "─".repeat(60));
            app.push(EventKind::Thought, "Available Models:");
            app.push(EventKind::Divider, "─".repeat(40));
            let models = [
                ("Groq",       "llama-3.3-70b-versatile", "Free tier ⚡ Very fast"),
                ("Groq",       "llama-3.1-8b-instant",    "Free tier ⚡ Ultra fast"),
                ("Groq",       "mixtral-8x7b-32768",      "Free tier · Long context"),
                ("OpenAI",     "gpt-4o",                  "Paid · Best quality"),
                ("OpenAI",     "gpt-4o-mini",             "Paid · Fast + cheap"),
                ("Anthropic",  "claude-sonnet-4-5",       "Paid · Best reasoning"),
                ("OpenRouter", "mistral-7b",              "Free tier · Open weights"),
                ("Ollama",     "llama3.1",                "Local · Fully private"),
                ("Ollama",     "mistral",                 "Local · Fast local"),
            ];
            for (provider, model, note) in &models {
                let active = if provider.to_lowercase() == app.provider.to_lowercase()
                             && model.contains(&app.model[..app.model.len().min(10)]) {
                    " ◀ active"
                } else { "" };
                app.push(
                    EventKind::ToolCall,
                    format!("{:<12} {:<35} {}{}", provider, model, note, active),
                );
            }
            app.push(EventKind::Thought, "Switch with: microdragon config provider <name>");
            app.push(EventKind::Divider, "─".repeat(60));
        }

        "/agents" => {
            app.push(EventKind::Divider, "─".repeat(60));
            app.push(EventKind::Thought, "🐉  MICRODRAGON — Agent Capabilities");
            app.push(EventKind::Divider, "─".repeat(60));
            let agents = [
                ("Research Agent",    "Web search (Brave/DDG) + synthesis + citations"),
                ("Code Agent",        "Generate, debug, review, test — any language"),
                ("Market Agent",      "Live market analysis, signals, risk assessment"),
                ("Automation Agent",  "Playwright browser + PyAutoGUI desktop scripts"),
                ("Memory Agent",      "Cross-session SQLite memory + context recall"),
                ("File Agent",        "Read, analyse, and reason over any code file"),
                ("Reflection Engine", "Self-critiques every response, refines if < 7/10"),
            ];
            for (name, cap) in &agents {
                app.push(EventKind::Done, format!("{:<22} {}", name, cap));
            }
            app.push(EventKind::Divider, "─".repeat(60));
            app.push(EventKind::Thought, "Pipeline: 9-phase PERCEIVE→PLAN→GATHER→SYNTHESIZE→REASON→REFLECT→REFINE→FORMAT→COMMIT");
            app.push(EventKind::Divider, "─".repeat(60));
        }

        "/memory" => {
            app.push(EventKind::Divider, "─".repeat(60));
            app.push(EventKind::Thought, "Recent memory (last 5 interactions):");
            // Collect memory BEFORE any mutable borrow of app
            let memory_lines: Vec<String> = {
                let mem = app.engine.memory.read().await;
                match mem.get_recent_context(10).await {
                    Ok(ctx) => {
                        ctx.iter().rev().take(5).map(|msg| {
                            let role = format!("{:?}", msg.role).to_lowercase();
                            let preview: String = msg.content.chars().take(80).collect();
                            format!("[{}] {}…", role, preview)
                        }).collect()
                    }
                    Err(_) => vec!["Error reading memory.".to_string()],
                }
            }; // lock released here
            if memory_lines.is_empty() {
                app.push(EventKind::Thought, "No memory yet. Start a conversation!");
            } else {
                for line in memory_lines {
                    app.push(EventKind::MemoryRecall, line);
                }
            }
            app.push(EventKind::Divider, "─".repeat(60));
        }

        "/provider" => {
            app.push(EventKind::Done, format!("Active provider: {}  ·  Model: {}", app.provider, app.model));
            app.push(EventKind::Thought, "To switch: microdragon config provider <name>");
        }

        cmd if cmd.starts_with("/research ") => {
            let query = cmd.trim_start_matches("/research ").trim().to_string();
            if !query.is_empty() {
                submit_task(app, query).await;
            }
        }

        cmd if cmd.starts_with("/code ") => {
            let task = cmd.trim_start_matches("/code ").trim().to_string();
            if !task.is_empty() {
                submit_task(app, format!("Generate code: {}", task)).await;
            }
        }

        _ => {
            app.push(EventKind::Warning, format!("Unknown command: {}  —  type /help for commands", cmd));
        }
    }
}

// ─── Rendering ────────────────────────────────────────────────────────────────

fn draw(f: &mut Frame, app: &mut App) {
    let area = f.area();

    // Fill background
    f.render_widget(
        Block::default().style(Style::default().bg(C_BG)),
        area,
    );

    // Show command palette overlay if active
    if app.mode == AppMode::CommandPalette {
        draw_palette(f, app, area);
        return;
    }

    // Main layout: header / log / input / shortcuts
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),   // header
            Constraint::Min(10),     // log
            Constraint::Length(3),   // input
            Constraint::Length(1),   // shortcuts
        ])
        .split(area);

    draw_header(f, app, chunks[0]);
    draw_log(f, app, chunks[1]);
    draw_input(f, app, chunks[2]);
    draw_shortcuts(f, app, chunks[3]);
}

fn draw_header(f: &mut Frame, app: &App, area: Rect) {
    let title = app.task_title.as_deref().unwrap_or("MICRODRAGON — Ready");
    let token_str = if app.total_tokens > 0 {
        format!("{}  {}", app.total_tokens, app.cost_str())
    } else {
        String::new()
    };

    let working_indicator = if app.mode == AppMode::Working {
        format!("  {}", THINKING_FRAMES[app.thinking_frame])
    } else {
        String::new()
    };

    let title_line = Line::from(vec![
        Span::styled("  # ", Style::default().fg(C_EMBER).add_modifier(Modifier::BOLD)),
        Span::styled(title, Style::default().fg(C_TITLE).add_modifier(Modifier::BOLD)),
        Span::styled(working_indicator, Style::default().fg(C_DIM)),
    ]);

    let right_text = if token_str.is_empty() {
        Line::from(Span::raw(""))
    } else {
        Line::from(vec![
            Span::styled(&token_str, Style::default().fg(C_DIM)),
            Span::raw("  "),
        ])
    };

    // Header block
    let header = Paragraph::new(title_line)
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(C_BORDER))
                .border_type(BorderType::Plain)
                .style(Style::default().bg(C_SURFACE))
        );
    f.render_widget(header, area);

    // Token count right-aligned (hack: render as separate widget over the header)
    if !right_text.spans.is_empty() {
        let token_area = Rect {
            x: area.x + area.width.saturating_sub(20),
            y: area.y + 1,
            width: 20,
            height: 1,
        };
        let tokens_p = Paragraph::new(right_text)
            .style(Style::default().bg(C_SURFACE))
            .alignment(Alignment::Right);
        f.render_widget(tokens_p, token_area);
    }
}

fn draw_log(f: &mut Frame, app: &mut App, area: Rect) {
    let inner_height = area.height.saturating_sub(2) as usize;
    let total = app.log.len();

    // Calculate visible window
    let scroll = if total <= inner_height {
        0
    } else {
        app.log_scroll.min(total - inner_height)
    };

    // Build list items for visible window
    let items: Vec<ListItem> = app.log.iter()
        .skip(scroll)
        .take(inner_height + 2)
        .map(|ev| render_event(ev))
        .collect();

    let log_list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::NONE)
                .style(Style::default().bg(C_BG))
        );

    app.log_list_state.select(None);
    f.render_stateful_widget(log_list, area, &mut app.log_list_state);
}

fn render_event(ev: &AgentEvent) -> ListItem<'static> {
    let (prefix_color, text_color, bold) = match ev.kind {
        EventKind::Thought      => (C_DIM,    C_WHITE,  false),
        EventKind::ToolCall     => (C_CYAN,   C_CYAN,   false),
        EventKind::FileRead     => (C_BLUE,   C_BLUE,   false),
        EventKind::WebSearch    => (C_YELLOW, C_YELLOW, false),
        EventKind::MemoryRecall => (C_PURPLE, C_PURPLE, false),
        EventKind::PhaseStart   => (C_GREEN,  C_WHITE,  false),
        EventKind::Done         => (C_GREEN,  C_GREEN,  true),
        EventKind::Warning      => (C_EMBER,  C_EMBER,  false),
        EventKind::Error        => (C_FIRE,   C_FIRE,   true),
        EventKind::Response     => (C_DIM,    C_WHITE,  false),
        EventKind::Divider      => (C_BORDER, C_BORDER, false),
        EventKind::UserInput    => (C_EMBER,  C_TITLE,  true),
    };

    let prefix = ev.prefix();
    let text_style = if bold {
        Style::default().fg(text_color).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(text_color)
    };

    // Special rendering for divider
    if ev.kind == EventKind::Divider {
        return ListItem::new(Line::from(
            Span::styled(format!("  {}", ev.text), Style::default().fg(C_BORDER))
        ));
    }

    // Normal event line
    let line = Line::from(vec![
        Span::styled(prefix, Style::default().fg(prefix_color).add_modifier(Modifier::BOLD)),
        Span::styled(" ", Style::default()),
        Span::styled(ev.text.clone(), text_style),
    ]);

    ListItem::new(line)
}

fn draw_input(f: &mut Frame, app: &App, area: Rect) {
    let is_working = app.mode == AppMode::Working;

    let prompt = if is_working {
        Span::styled("  ⠿ ", Style::default().fg(C_GREEN).add_modifier(Modifier::BOLD))
    } else {
        Span::styled("  ❯ ", Style::default().fg(C_GREEN).add_modifier(Modifier::BOLD))
    };

    let input_display = if is_working {
        Span::styled("Working… (esc to interrupt)", Style::default().fg(C_DIM))
    } else if app.input.is_empty() {
        Span::styled("Type your task or / for commands…", Style::default().fg(C_DIM))
    } else {
        Span::styled(app.input.clone(), Style::default().fg(C_WHITE))
    };

    let input_line = Line::from(vec![prompt, input_display]);

    let input_p = Paragraph::new(input_line)
        .block(
            Block::default()
                .borders(Borders::TOP | Borders::BOTTOM)
                .border_style(Style::default().fg(C_BORDER))
                .border_type(BorderType::Plain)
                .style(Style::default().bg(C_SURFACE))
        );

    f.render_widget(input_p, area);

    // Show cursor position
    if !is_working {
        let cursor_x = area.x + 5 + app.input_cursor.min(area.width as usize - 6) as u16;
        let cursor_y = area.y + 1;
        f.set_cursor_position((cursor_x, cursor_y));
    }
}

fn draw_shortcuts(f: &mut Frame, app: &App, area: Rect) {
    let build_status = if app.mode == AppMode::Working {
        Span::styled(
            format!("  {} Build  · {}  ", THINKING_FRAMES[app.thinking_frame], app.model),
            Style::default().fg(C_BG).bg(C_GREEN).add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled(
            format!("  Build  · {} ", app.model),
            Style::default().fg(C_GREEN).add_modifier(Modifier::BOLD),
        )
    };

    let line = Line::from(vec![
        build_status,
        Span::styled(format!("  {}  ", app.provider), Style::default().fg(C_DIM)),
        Span::styled("ctrl+p", Style::default().fg(C_WHITE).add_modifier(Modifier::BOLD)),
        Span::styled(" commands  ", Style::default().fg(C_DIM)),
        Span::styled("tab", Style::default().fg(C_WHITE).add_modifier(Modifier::BOLD)),
        Span::styled(" agents  ", Style::default().fg(C_DIM)),
        Span::styled("ctrl+c", Style::default().fg(C_WHITE).add_modifier(Modifier::BOLD)),
        Span::styled(" quit  ", Style::default().fg(C_DIM)),
        Span::styled("esc", Style::default().fg(C_WHITE).add_modifier(Modifier::BOLD)),
        Span::styled(" interrupt", Style::default().fg(C_DIM)),
    ]);

    f.render_widget(
        Paragraph::new(line).style(Style::default().bg(C_SURFACE)),
        area,
    );
}

// ─── Command Palette overlay ──────────────────────────────────────────────────

fn draw_palette(f: &mut Frame, app: &App, area: Rect) {
    // Darken background by rendering the regular UI first then overlaying
    draw_header(f, app, Rect { x: area.x, y: area.y, width: area.width, height: 3 });

    // Palette box — centered, 60 wide
    let palette_w = 64u16.min(area.width.saturating_sub(4));
    let palette_h = (SLASH_COMMANDS.len() as u16 + 4).min(area.height - 6);
    let palette_x = (area.width.saturating_sub(palette_w)) / 2;
    let palette_y = 4u16;

    let palette_area = Rect {
        x: area.x + palette_x,
        y: area.y + palette_y,
        width: palette_w,
        height: palette_h,
    };

    // Background
    f.render_widget(
        Block::default()
            .style(Style::default().bg(C_SURFACE))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(C_GREEN))
            .title(Span::styled(" Commands ", Style::default().fg(C_GREEN).add_modifier(Modifier::BOLD))),
        palette_area,
    );

    // Filter input
    let filter_area = Rect {
        x: palette_area.x + 1,
        y: palette_area.y + 1,
        width: palette_area.width - 2,
        height: 1,
    };
    let filter_display = if app.palette_filter.is_empty() {
        Span::styled("Type to filter commands…", Style::default().fg(C_DIM))
    } else {
        Span::styled(app.palette_filter.clone(), Style::default().fg(C_WHITE))
    };
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("/ ", Style::default().fg(C_GREEN).add_modifier(Modifier::BOLD)),
            filter_display,
        ])),
        filter_area,
    );

    // Separator
    let sep_area = Rect { x: palette_area.x + 1, y: palette_area.y + 2, width: palette_area.width - 2, height: 1 };
    f.render_widget(
        Paragraph::new(Line::from(Span::styled("─".repeat((palette_area.width - 2) as usize), Style::default().fg(C_BORDER)))),
        sep_area,
    );

    // Command list
    let list_area = Rect {
        x: palette_area.x + 1,
        y: palette_area.y + 3,
        width: palette_area.width - 2,
        height: palette_area.height - 4,
    };

    let cmds = app.visible_commands();
    let items: Vec<ListItem> = cmds.iter().enumerate().map(|(i, cmd)| {
        let is_selected = i == app.palette_index;
        let style = if is_selected {
            Style::default().fg(C_BG).bg(C_GREEN).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(C_WHITE).bg(C_SURFACE)
        };
        let name_style = if is_selected {
            Style::default().fg(C_BG).bg(C_GREEN).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(C_GREEN).bg(C_SURFACE).add_modifier(Modifier::BOLD)
        };
        ListItem::new(Line::from(vec![
            Span::styled(format!("  {:<14}", cmd.name), name_style),
            Span::styled(cmd.description, style),
        ]))
    }).collect();

    let mut list_state = ListState::default();
    list_state.select(Some(app.palette_index));

    f.render_stateful_widget(
        List::new(items).highlight_style(Style::default().bg(C_GREEN)),
        list_area,
        &mut list_state,
    );

    // Footer
    let footer_area = Rect {
        x: palette_area.x + 1,
        y: palette_area.y + palette_area.height - 1,
        width: palette_area.width - 2,
        height: 1,
    };
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("↑↓ navigate  ", Style::default().fg(C_DIM)),
            Span::styled("enter", Style::default().fg(C_GREEN).add_modifier(Modifier::BOLD)),
            Span::styled(" select  ", Style::default().fg(C_DIM)),
            Span::styled("esc", Style::default().fg(C_GREEN).add_modifier(Modifier::BOLD)),
            Span::styled(" close", Style::default().fg(C_DIM)),
        ])),
        footer_area,
    );
}
