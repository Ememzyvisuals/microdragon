// microdragon-core/src/cli/interactive/mod.rs
// MICRODRAGON Interactive Mode — persistent shell-like AI session
// Features: command history, input editing, context awareness, streaming

pub mod history;
pub mod input;
pub mod prompt;

pub use history::CommandHistory;
pub use input::LineEditor;
pub use prompt::PromptRenderer;

use std::sync::Arc;
use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor, Attribute, SetAttribute},
    cursor, terminal,
};
use std::io::{self, Write};

use crate::engine::MicrodragonEngine;
use crate::cli::animation::{Spinner, SpinnerStyle, StatusLine, StatusState, Typewriter};
use crate::cli::terminal as term;

const WELCOME_RICH: &str = r#"
  ╔═══════════════════════════════════════════════════════╗
  ║  MICRODRAGON — Universal AI Agent  •  Interactive Mode       ║
  ║  Type your task or question. Ctrl+C to exit.          ║
  ║  Commands: /help /status /clear /history /exit        ║
  ╚═══════════════════════════════════════════════════════╝
"#;

const WELCOME_SIMPLE: &str = r#"
MICRODRAGON — Universal AI Agent (Interactive Mode)
Type your task or question. Ctrl+C or /exit to quit.
Commands: /help /status /clear /history
"#;

pub struct InteractiveMode {
    engine: Arc<MicrodragonEngine>,
    history: CommandHistory,
    editor: LineEditor,
    session_turns: u32,
}

impl InteractiveMode {
    pub fn new(engine: Arc<MicrodragonEngine>) -> Self {
        Self {
            engine,
            history: CommandHistory::new(1000),
            editor: LineEditor::new(),
            session_turns: 0,
        }
    }

    pub async fn run(mut self) -> Result<()> {
        // Print welcome banner
        self.print_welcome();

        // Check configuration
        let config = self.engine.get_config().await;
        if !config.is_configured() {
            self.print_warning("No AI provider configured. Run 'microdragon setup' first.");
        } else {
            self.print_status_bar(&config.ai.active_provider.to_string(),
                                   &config.ai.providers.get_model(&config.ai.active_provider));
        }

        // Main REPL loop
        loop {
            let input = match self.read_input().await {
                Ok(Some(line)) => line.trim().to_string(),
                Ok(None) => break,       // EOF / Ctrl+D
                Err(_) => continue,
            };

            if input.is_empty() { continue; }

            // Store in history
            self.history.push(input.clone());

            // Handle built-in commands
            match self.handle_builtin(&input).await {
                BuiltinResult::Exit => break,
                BuiltinResult::Handled => continue,
                BuiltinResult::PassThrough => {}
            }

            // Process through AI engine
            self.session_turns += 1;
            if let Err(e) = self.process_input(&input).await {
                self.print_error(&format!("{}", e));
            }
        }

        // Clean exit
        self.print_goodbye();
        Ok(())
    }

    // ─── Input reading ────────────────────────────────────────────────────

    async fn read_input(&mut self) -> Result<Option<String>> {
        self.editor.readline(&self.make_prompt()).await
    }

    fn make_prompt(&self) -> String {
        if term::supports_color() {
            // Rich prompt with turn counter
            format!("\x1b[36mmicrodragon\x1b[0m\x1b[90m[{}]\x1b[0m\x1b[36m ▸ \x1b[0m", self.session_turns + 1)
        } else {
            format!("microdragon[{}] > ", self.session_turns + 1)
        }
    }

    // ─── Built-in command handling ────────────────────────────────────────

    async fn handle_builtin(&self, input: &str) -> BuiltinResult {
        match input {
            "/exit" | "/quit" | "exit" | "quit" => BuiltinResult::Exit,

            "/clear" | "/c" => {
                if term::is_rich() {
                    let _ = execute!(io::stdout(), terminal::Clear(terminal::ClearType::All), cursor::MoveTo(0, 0));
                } else {
                    for _ in 0..40 { println!(); }
                }
                BuiltinResult::Handled
            }

            "/history" | "/hist" => {
                self.show_history();
                BuiltinResult::Handled
            }

            "/status" | "/stat" => {
                let health = self.engine.health_check().await;
                self.print_status_bar(&health.provider, &health.model);
                BuiltinResult::Handled
            }

            "/help" | "help" | "?" => {
                self.print_help();
                BuiltinResult::Handled
            }

            s if s.starts_with("/set ") => {
                // /set temperature 0.5 etc.
                self.handle_set_command(&s[5..]).await;
                BuiltinResult::Handled
            }

            _ => BuiltinResult::PassThrough,
        }
    }

    async fn handle_set_command(&self, args: &str) {
        let parts: Vec<&str> = args.splitn(2, ' ').collect();
        if parts.len() < 2 { self.print_error("Usage: /set <key> <value>"); return; }
        match parts[0] {
            "provider" => {
                let mut cfg = self.engine.get_config().await;
                use crate::config::providers::ModelProvider;
                cfg.ai.active_provider = match parts[1] {
                    "anthropic" => ModelProvider::Anthropic,
                    "openai"    => ModelProvider::OpenAI,
                    "groq"      => ModelProvider::Groq,
                    "openrouter"=> ModelProvider::OpenRouter,
                    "custom"    => ModelProvider::Custom,
                    other => { self.print_error(&format!("Unknown provider: {}", other)); return; }
                };
                match self.engine.update_config(cfg).await {
                    Ok(_) => self.print_success(&format!("Provider set to {}", parts[1])),
                    Err(e) => self.print_error(&format!("{}", e)),
                }
            }
            other => self.print_error(&format!("Unknown setting: {}", other)),
        }
    }

    // ─── AI processing ────────────────────────────────────────────────────

    async fn process_input(&self, input: &str) -> Result<()> {
        // Show status transition: thinking → executing
        let mut status = StatusLine::new(StatusState::Thinking);

        // Determine if we should stream
        let config = self.engine.get_config().await;
        let use_streaming = config.ai.active_provider.supports_streaming();

        if use_streaming {
            status.transition(StatusState::Executing);

            // Create streaming channel
            let (tx, rx) = tokio::sync::mpsc::channel::<String>(256);

            let engine = self.engine.clone();
            let input_owned = input.to_string();

            // Spawn streaming task
            tokio::spawn(async move {
                if let Err(e) = engine.process_streaming(&input_owned, tx).await {
                    eprintln!("\nStream error: {}", e);
                }
            });

            // Consume stream with typewriter
            let writer = Typewriter::new();
            writer.print_response_prefix();
            let full = writer.stream_tokens(rx).await;

            // Print metadata footer
            self.print_footer();

        } else {
            // Non-streaming: spinner + full response
            status.transition(StatusState::Executing);

            let result = self.engine.process_command(input).await?;

            // Clear spinner line, then print
            status.complete("");

            // Print AI prefix
            if term::supports_color() {
                let _ = execute!(
                    io::stdout(),
                    SetForegroundColor(Color::DarkGrey),
                    Print("MICRODRAGON ▸ "),
                    ResetColor,
                );
            } else {
                print!("MICRODRAGON > ");
            }

            println!("{}\n", result.response);
            self.print_meta_footer(result.tokens_used, result.latency_ms, &result.model);
        }

        Ok(())
    }

    // ─── Display helpers ──────────────────────────────────────────────────

    fn print_welcome(&self) {
        if term::is_rich() && term::supports_color() {
            let _ = execute!(
                io::stdout(),
                SetForegroundColor(Color::Cyan),
                Print(WELCOME_RICH),
                ResetColor,
            );
        } else {
            println!("{}", WELCOME_SIMPLE);
        }
    }

    fn print_status_bar(&self, provider: &str, model: &str) {
        if term::supports_color() {
            let _ = execute!(
                io::stdout(),
                SetForegroundColor(Color::DarkGrey),
                Print(format!("  Provider: ")),
                ResetColor,
                SetForegroundColor(Color::Cyan),
                Print(provider),
                ResetColor,
                SetForegroundColor(Color::DarkGrey),
                Print("  •  Model: "),
                ResetColor,
                SetForegroundColor(Color::Cyan),
                Print(model),
                ResetColor,
                Print("\n\n"),
            );
        } else {
            println!("  Provider: {}  Model: {}\n", provider, model);
        }
    }

    fn print_footer(&self) {
        if term::supports_color() {
            let _ = execute!(
                io::stdout(),
                SetForegroundColor(Color::DarkGrey),
                SetAttribute(Attribute::Dim),
                Print("  ─────────────────────────────────────────\n"),
                SetAttribute(Attribute::Reset),
                ResetColor,
            );
        }
    }

    fn print_meta_footer(&self, tokens: u32, latency_ms: u64, model: &str) {
        if term::supports_color() {
            let _ = execute!(
                io::stdout(),
                SetForegroundColor(Color::DarkGrey),
                SetAttribute(Attribute::Dim),
                Print(format!("  {} tokens • {}ms • {}\n", tokens, latency_ms, model)),
                SetAttribute(Attribute::Reset),
                ResetColor,
            );
        } else {
            println!("  [{} tokens, {}ms]", tokens, latency_ms);
        }
    }

    fn print_success(&self, msg: &str) {
        let prefix = if term::supports_unicode() { "✓ " } else { "[OK] " };
        if term::supports_color() {
            let _ = execute!(io::stdout(), SetForegroundColor(Color::Green), Print(format!("{}{}\n", prefix, msg)), ResetColor);
        } else {
            println!("{}{}", prefix, msg);
        }
    }

    fn print_error(&self, msg: &str) {
        let prefix = if term::supports_unicode() { "✗ " } else { "[ERR] " };
        if term::supports_color() {
            let _ = execute!(io::stdout(), SetForegroundColor(Color::Red), Print(format!("{}{}\n", prefix, msg)), ResetColor);
        } else {
            eprintln!("{}{}", prefix, msg);
        }
    }

    fn print_warning(&self, msg: &str) {
        let prefix = if term::supports_unicode() { "⚠ " } else { "[WARN] " };
        if term::supports_color() {
            let _ = execute!(io::stdout(), SetForegroundColor(Color::Yellow), Print(format!("{}{}\n", prefix, msg)), ResetColor);
        } else {
            println!("{}{}", prefix, msg);
        }
    }

    fn print_help(&self) {
        let help = if term::is_rich() {
            format!(
                r#"
  {}
  {}                  Ask anything or give a task
  {}        Show conversation history
  {}        Show system status
  {}          Switch provider: anthropic, openai, groq
  {}          Clear the screen
  {}          Exit interactive mode
  {}                   Show this help
"#,
                if term::supports_color() { "\x1b[1;36mCommands\x1b[0m" } else { "Commands" },
                if term::supports_color() { "\x1b[33m[any text]\x1b[0m    " } else { "[any text]    " },
                if term::supports_color() { "\x1b[33m/history\x1b[0m       " } else { "/history       " },
                if term::supports_color() { "\x1b[33m/status\x1b[0m        " } else { "/status        " },
                if term::supports_color() { "\x1b[33m/set provider\x1b[0m  " } else { "/set provider  " },
                if term::supports_color() { "\x1b[33m/clear\x1b[0m         " } else { "/clear         " },
                if term::supports_color() { "\x1b[33m/exit\x1b[0m          " } else { "/exit          " },
                if term::supports_color() { "\x1b[33m/help\x1b[0m          " } else { "/help          " },
            )
        } else {
            "Commands: /history /status /set provider /clear /exit /help\nType anything to chat with MICRODRAGON.".to_string()
        };
        println!("{}", help);
    }

    fn show_history(&self) {
        let entries = self.history.recent(20);
        if entries.is_empty() {
            println!("No history yet.");
            return;
        }
        println!();
        for (i, entry) in entries.iter().enumerate() {
            let preview = &entry[..entry.len().min(80)];
            if term::supports_color() {
                let _ = execute!(
                    io::stdout(),
                    SetForegroundColor(Color::DarkGrey),
                    Print(format!("  {:3}  ", entries.len() - i)),
                    ResetColor,
                    Print(preview),
                    Print("\n"),
                );
            } else {
                println!("  {:3}  {}", entries.len() - i, preview);
            }
        }
        println!();
    }

    fn print_goodbye(&self) {
        if term::supports_color() {
            let _ = execute!(
                io::stdout(),
                SetForegroundColor(Color::Cyan),
                SetAttribute(Attribute::Dim),
                Print("\n  Goodbye. MICRODRAGON session ended.\n\n"),
                SetAttribute(Attribute::Reset),
                ResetColor,
            );
        } else {
            println!("\nGoodbye.");
        }
    }
}

enum BuiltinResult {
    Exit,
    Handled,
    PassThrough,
}
