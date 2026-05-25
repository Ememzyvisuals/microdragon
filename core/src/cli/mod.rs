// microdragon-core/src/cli/mod.rs
// MICRODRAGON CLI - Primary Interface

pub mod commands;
pub mod interactive;
pub mod display;
pub mod setup;
pub mod terminal;
pub mod theme;
pub mod animation;
pub mod stream_renderer;
pub mod first_launch;
pub mod simple_mode;
pub mod markdown;
pub mod tui;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::sync::Arc;

use crate::engine::MicrodragonEngine;
use self::interactive::InteractiveMode;
use self::setup::SetupWizard;
use self::display::{print_ok, print_warn, Spinner, response_footer};
use self::theme::Theme;
use crossterm::style::Stylize;

#[derive(Parser)]
#[command(
    name = "microdragon",
    version = "0.1.0",
    author = "MICRODRAGON AI",
    about = "Universal Human-Level AI Agent",
    long_about = "MICRODRAGON is a production-grade AI agent for personal, business, and technical tasks.\nRun 'microdragon setup' to configure your AI provider.",
    propagate_version = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Output format (text, json, markdown)
    #[arg(short, long, global = true, default_value = "text")]
    pub output: String,

    /// Skip confirmation prompts
    #[arg(short, long, global = true)]
    pub yes: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Interactive conversation mode (default)
    #[command(alias = "i")]
    Interactive,

    /// Ask MICRODRAGON a question or give it a task
    #[command(alias = "q")]
    Ask {
        /// The prompt or task description
        prompt: Vec<String>,
        /// Stream the response
        #[arg(short, long)]
        stream: bool,
    },

    /// Code generation and management
    #[command(alias = "c")]
    Code {
        #[command(subcommand)]
        action: CodeCommands,
    },

    /// Research and web intelligence
    #[command(alias = "r")]
    Research {
        /// Topic or query to research
        query: Vec<String>,
        /// Number of sources to gather
        #[arg(short, long, default_value = "5")]
        sources: u32,
        /// Output to file
        #[arg(short, long)]
        output_file: Option<String>,
    },

    /// Automation tasks
    #[command(alias = "auto")]
    Automate {
        #[command(subcommand)]
        action: AutomateCommands,
    },

    /// Social platform management
    #[command(alias = "soc")]
    Social {
        #[command(subcommand)]
        action: SocialCommands,
    },

    /// Business and trading tools
    #[command(alias = "biz")]
    Business {
        #[command(subcommand)]
        action: BusinessCommands,
    },

    /// Task and memory management
    #[command(alias = "t")]
    Tasks {
        #[command(subcommand)]
        action: TaskCommands,
    },

    /// Configure MICRODRAGON
    #[command(alias = "cfg")]
    Config {
        #[command(subcommand)]
        action: ConfigCommands,
    },

    /// Run initial setup wizard
    Setup,

    /// Show system status and health
    Status,

    /// Clear conversation history
    Clear,

    /// Create real documents — Word, Excel, PDF, PowerPoint
    #[command(alias = "doc")]
    Document {
        #[command(subcommand)]
        action: DocumentCommands,
    },

    /// Design — HTML pages, SVG graphics, UI components
    #[command(alias = "des")]
    Design {
        #[command(subcommand)]
        action: DesignCommands,
    },

    /// File system — navigate, read, create files and folders
    #[command(alias = "fs")]
    Files {
        #[command(subcommand)]
        action: FilesCommands,
    },

    /// Voice — speak and listen
    Voice {
        #[command(subcommand)]
        action: VoiceCommands,
    },

    /// Gaming — MICRODRAGON plays games autonomously
    #[command(alias = "g")]
    Game {
        #[command(subcommand)]
        action: GameCommands,
    },

    /// GitHub — PR review, issues, repo management
    #[command(alias = "gh")]
    Github {
        #[command(subcommand)]
        action: GithubCommands,
    },

    /// Open an app or file with its default application
    Open {
        target: String,
    },

    /// Show version information
    Version,
}

#[derive(Subcommand)]
pub enum CodeCommands {
    /// Generate code from description
    Generate {
        description: Vec<String>,
        #[arg(short, long)]
        language: Option<String>,
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Debug a file or code snippet
    Debug {
        file: String,
        #[arg(short, long)]
        language: Option<String>,
    },
    /// Review code quality
    Review {
        file: String,
    },
    /// Run tests
    Test {
        #[arg(short, long)]
        path: Option<String>,
    },
    /// Git operations via AI
    Git {
        action: String,
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
}

#[derive(Subcommand)]
pub enum AutomateCommands {
    /// Automate browser tasks
    Browser {
        task: Vec<String>,
        #[arg(short, long)]
        url: Option<String>,
        #[arg(short, long)]
        headless: bool,
    },
    /// Automate desktop tasks
    Desktop {
        task: Vec<String>,
    },
    /// Run a workflow file
    Run {
        workflow: String,
        #[arg(short, long)]
        vars: Vec<String>,
    },
    /// Schedule a recurring task
    Schedule {
        task: Vec<String>,
        #[arg(short, long)]
        cron: String,
    },
    /// List scheduled tasks
    List,
}

#[derive(Subcommand)]
pub enum SocialCommands {
    /// Start WhatsApp listener
    Whatsapp {
        #[command(subcommand)]
        action: WhatsappCommands,
    },
    /// Telegram bot management
    Telegram {
        #[command(subcommand)]
        action: TelegramCommands,
    },
    /// Discord bot management
    Discord {
        #[command(subcommand)]
        action: DiscordCommands,
    },
    /// Send a message to a platform
    Send {
        platform: String,
        recipient: String,
        message: Vec<String>,
    },
}

#[derive(Subcommand)]
pub enum WhatsappCommands {
    /// Start WhatsApp bridge
    Start,
    /// Stop WhatsApp bridge
    Stop,
    /// Show QR code for login
    Qr,
    /// Show recent messages
    Messages { #[arg(short, long, default_value = "20")] limit: u32 },
}

#[derive(Subcommand)]
pub enum TelegramCommands {
    Start,
    Stop,
    Status,
    Messages { #[arg(short, long, default_value = "20")] limit: u32 },
}

#[derive(Subcommand)]
pub enum DiscordCommands {
    Start,
    Stop,
    Status,
    Messages { #[arg(short, long, default_value = "20")] limit: u32 },
}

#[derive(Subcommand)]
pub enum BusinessCommands {
    /// Market analysis
    Market {
        symbol: String,
        #[arg(short, long, default_value = "1d")]
        interval: String,
    },
    /// Portfolio overview
    Portfolio,
    /// Risk analysis
    Risk {
        symbol: String,
    },
}

#[derive(Subcommand)]
pub enum TaskCommands {
    /// List all tasks
    List {
        #[arg(short, long)]
        status: Option<String>,
    },
    /// Show task details
    Show { id: String },
    /// Cancel a task
    Cancel { id: String },
    /// Show conversation history
    History {
        #[arg(short, long, default_value = "20")]
        limit: u32,
    },

    /// Export history
    Export {
        #[arg(short, long)]
        format: Option<String>,
        #[arg(short, long)]
        output: Option<String>,
    },
}

// ─── New command enums ────────────────────────────────────────────────────────

#[derive(Subcommand)]
pub enum DocumentCommands {
    /// Create a Word document (.docx)
    Word {
        /// What to create (title or description)
        description: Vec<String>,
        /// Output file path (default: Desktop/document.docx)
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Create an Excel spreadsheet (.xlsx)
    Excel {
        description: Vec<String>,
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Create a PDF document
    Pdf {
        description: Vec<String>,
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Create a PowerPoint presentation (.pptx)
    Pptx {
        description: Vec<String>,
        #[arg(short, long)]
        output: Option<String>,
        #[arg(short, long, default_value = "5")]
        slides: u32,
    },
    /// Read any document (PDF, DOCX, XLSX, TXT)
    Read {
        path: String,
    },
    /// Install document creation dependencies
    Install,
}

#[derive(Subcommand)]
pub enum DesignCommands {
    /// Generate an HTML/CSS page or component
    Html {
        description: Vec<String>,
        #[arg(short, long)]
        output: Option<String>,
        #[arg(long, default_value = "modern")]
        style: String,
    },
    /// Generate an SVG graphic
    Svg {
        description: Vec<String>,
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Create an image using Pillow (programmatic graphic design)
    Image {
        description: Vec<String>,
        #[arg(short, long)]
        output: Option<String>,
        #[arg(long, default_value = "1920")]
        width: u32,
        #[arg(long, default_value = "1080")]
        height: u32,
    },
    /// Open a design file in its default application
    Open {
        path: String,
    },
}

#[derive(Subcommand)]
pub enum FilesCommands {
    /// List directory contents
    Ls {
        /// Path to list (default: current directory)
        path: Option<String>,
    },
    /// Read a file
    Read {
        path: String,
    },
    /// Write content to a file
    Write {
        path: String,
        #[arg(short, long)]
        content: Option<String>,
    },
    /// Create a directory
    Mkdir {
        path: String,
    },
    /// Find files matching a pattern
    Find {
        /// Root directory to search
        root: Option<String>,
        /// Filename pattern or extension
        pattern: String,
    },
    /// Show PC file system overview (home, desktop, documents paths)
    Overview,
    /// Navigate to a directory and list it
    Cd {
        path: String,
    },
}

#[derive(Subcommand)]
pub enum VoiceCommands {
    /// Speak text aloud using TTS
    Say {
        text: Vec<String>,
    },
    /// Listen to microphone and transcribe
    Listen {
        #[arg(short, long, default_value = "5")]
        duration: u32,
    },
    /// Show voice setup information
    Setup,
    /// Install voice dependencies
    Install,
}

#[derive(Subcommand)]
pub enum GameCommands {
    /// Start MICRODRAGON playing a game
    Play {
        /// Game name (e.g. "GTA V", "Mortal Kombat 11", "Need for Speed Heat")
        game: Vec<String>,
        /// Duration in seconds (default: 300)
        #[arg(short, long, default_value = "300")]
        duration: u32,
    },
    /// Stop active game session
    Stop,
    /// Show game session status and supported games
    Status,
    /// Install gaming dependencies (mss, pynput, opencv)
    Install,
}

#[derive(Subcommand)]
pub enum GithubCommands {
    /// Review a pull request
    Review {
        /// PR URL (e.g. https://github.com/owner/repo/pull/123)
        pr_url: String,
        /// Post the review to GitHub (requires GITHUB_TOKEN)
        #[arg(short, long)]
        post: bool,
    },
    /// Get repository overview
    Repo {
        /// owner/repo format
        repo: String,
    },
    /// Create an issue
    Issue {
        repo: String,
        title: Vec<String>,
    },
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Show current configuration
    Show,
    /// Set active AI provider
    Provider {
        /// Provider name (anthropic, openai, groq, openrouter, custom)
        name: String,
    },
    /// Set API key for a provider
    SetKey {
        provider: String,
        key: String,
    },
    /// Set AI model
    Model {
        model_name: String,
    },
    /// Reset to defaults
    Reset,
    /// Export config
    Export {
        path: String,
    },
}

pub struct MicrodragonCli {
    engine: Arc<MicrodragonEngine>,
}

impl MicrodragonCli {
    pub fn new(engine: Arc<MicrodragonEngine>) -> Self {
        Self { engine }
    }

    pub async fn run(self) -> Result<()> {
        let cli = Cli::parse();

        match cli.command {
            None | Some(Commands::Interactive) => {
                InteractiveMode::new(self.engine).run().await
            }

            Some(Commands::Ask { prompt, stream }) => {
                let text = prompt.join(" ");
                if stream {
                    self.run_streaming(&text).await
                } else {
                    self.run_ask(&text, &cli.output).await
                }
            }

            Some(Commands::Setup) => {
                SetupWizard::new(self.engine).run().await
            }

            Some(Commands::Status) => {
                self.show_status().await
            }

            Some(Commands::Config { action }) => {
                self.handle_config(action).await
            }

            Some(Commands::Code { action }) => {
                self.handle_code(action, &cli.output).await
            }

            Some(Commands::Research { query, sources, output_file }) => {
                self.handle_research(&query.join(" "), sources, output_file).await
            }

            Some(Commands::Tasks { action }) => {
                self.handle_tasks(action).await
            }

            Some(Commands::Social { action }) => {
                self.handle_social(action).await
            }

            Some(Commands::Business { action }) => {
                self.handle_business(action).await
            }

            Some(Commands::Automate { action }) => {
                self.handle_automate(action).await
            }

            Some(Commands::Clear) => {
                self.clear_history().await
            }

            Some(Commands::Version) => {
                self.show_version();
                Ok(())
            }

            // ── Document commands ────────────────────────────────────────────
            Some(Commands::Document { action }) => {
                self.handle_document(action).await
            }

            // ── Design commands ──────────────────────────────────────────────
            Some(Commands::Design { action }) => {
                self.handle_design(action).await
            }

            // ── Files commands ───────────────────────────────────────────────
            Some(Commands::Files { action }) => {
                self.handle_files(action).await
            }

            // ── Voice commands ───────────────────────────────────────────────
            Some(Commands::Voice { action }) => {
                self.handle_voice(action).await
            }

            // ── Game commands ────────────────────────────────────────────────
            Some(Commands::Game { action }) => {
                self.handle_game(action).await
            }

            // ── GitHub commands ──────────────────────────────────────────────
            Some(Commands::Github { action }) => {
                self.handle_github(action).await
            }

            // ── Open app/file ────────────────────────────────────────────────
            Some(Commands::Open { target }) => {
                self.handle_open(&target).await
            }
        }
    }

    async fn run_ask(&self, input: &str, output_format: &str) -> Result<()> {
        let config = self.engine.get_config().await;
        if !config.is_configured() {
            return self.offer_inline_setup("ask").await;
        }

        let spinner = Spinner::new("Thinking");
        let result = self.engine.process_command(input).await?;
        spinner.succeed("Done");

        match output_format {
            "json" => {
                let json = serde_json::json!({
                    "response": result.response,
                    "model": result.model,
                    "provider": result.provider,
                    "tokens": result.tokens_used,
                    "latency_ms": result.latency_ms
                });
                println!("{}", serde_json::to_string_pretty(&json)?);
            }
            _ => {
                markdown::render(&result.response);
                response_footer(&result.provider, &result.model, result.tokens_used, result.latency_ms);
            }
        }
        Ok(())
    }

    async fn run_streaming(&self, input: &str) -> Result<()> {
        let config = self.engine.get_config().await;
        if !config.is_configured() {
            return self.offer_inline_setup("streaming").await;
        }

        let (tx, rx) = tokio::sync::mpsc::channel::<String>(512);
        let engine = self.engine.clone();
        let input_owned = input.to_string();

        tokio::spawn(async move {
            engine.process_streaming(&input_owned, tx).await.ok();
        });

        let mut renderer = stream_renderer::StreamRenderer::new();
        renderer.render_stream(rx).await;
        Ok(())
    }

    async fn show_status(&self) -> Result<()> {
        use self::display::{section, kv, kv_colored};
        use crossterm::style::Color;

        let health = self.engine.health_check().await;
        let config = self.engine.get_config().await;

        section("MICRODRAGON Status");
        kv_colored("Engine:",   if health.is_healthy { "healthy" } else { "not configured" },
            if health.is_healthy { Color::Green } else { Color::Yellow });
        kv("Provider:",  &health.provider);
        kv("Model:",     &health.model);
        kv("Tasks:",     &health.active_tasks.to_string());
        kv("Memory:",    if health.memory_ok { "ok" } else { "error" });
        println!();
        kv_colored("WhatsApp:", if config.social.whatsapp_enabled { "enabled" } else { "disabled" },
            if config.social.whatsapp_enabled { Color::Green } else { Color::DarkGrey });
        kv_colored("Telegram:", if config.social.telegram_enabled { "enabled" } else { "disabled" },
            if config.social.telegram_enabled { Color::Green } else { Color::DarkGrey });
        kv_colored("Discord:",  if config.social.discord_enabled  { "enabled" } else { "disabled" },
            if config.social.discord_enabled  { Color::Green } else { Color::DarkGrey });
        println!();
        Ok(())
    }

    async fn handle_config(&self, action: ConfigCommands) -> Result<()> {
        let mut config = self.engine.get_config().await;

        match action {
            ConfigCommands::Show => {
                let display = toml::to_string_pretty(&config)?;
                // Mask API keys
                let masked = mask_api_keys(&display);
                println!("{}", masked);
            }

            ConfigCommands::Provider { name } => {
                use crate::config::providers::ModelProvider;
                config.ai.active_provider = match name.to_lowercase().as_str() {
                    "anthropic" | "claude" => ModelProvider::Anthropic,
                    "openai" | "gpt" => ModelProvider::OpenAI,
                    "groq" => ModelProvider::Groq,
                    "openrouter" => ModelProvider::OpenRouter,
                    "custom" | "local" | "ollama" => ModelProvider::Custom,
                    _ => {
                        eprintln!("{} Unknown provider: {}", Theme::error_str("✗"), name);
                        eprintln!("Valid: anthropic, openai, groq, openrouter, custom");
                        return Ok(());
                    }
                };
                self.engine.update_config(config).await?;
                println!("{} Provider set to: {}", Theme::success_str("✓"), name);
            }

            ConfigCommands::SetKey { provider, key } => {
                let provider_lower = provider.to_lowercase();
                match provider_lower.as_str() {
                    "anthropic" => {
                        config.ai.providers.anthropic_api_key = Some(key);
                        config.ai.active_provider = crate::config::providers::ModelProvider::Anthropic;
                    }
                    "openai" => {
                        config.ai.providers.openai_api_key = Some(key);
                        config.ai.active_provider = crate::config::providers::ModelProvider::OpenAI;
                    }
                    "groq" => {
                        config.ai.providers.groq_api_key = Some(key);
                        config.ai.active_provider = crate::config::providers::ModelProvider::Groq;
                    }
                    "openrouter" => {
                        config.ai.providers.openrouter_api_key = Some(key);
                        config.ai.active_provider = crate::config::providers::ModelProvider::OpenRouter;
                    }
                    "telegram" => {
                        config.social.telegram_bot_token = Some(key);
                        config.social.telegram_enabled = true;
                    }
                    "discord" => {
                        config.social.discord_bot_token = Some(key);
                        config.social.discord_enabled = true;
                    }
                    "github" => {
                        std::env::set_var("GITHUB_TOKEN", &key);
                        println!("{} GitHub token set (session only — add GITHUB_TOKEN to your .env for persistence)",
                            Theme::success_str("✓"));
                        return Ok(());
                    }
                    "brave" => {
                        std::env::set_var("BRAVE_SEARCH_API_KEY", &key);
                        println!("{} Brave Search key set (session only — add BRAVE_SEARCH_API_KEY to .env for persistence)",
                            Theme::success_str("✓"));
                        return Ok(());
                    }
                    _ => {
                        print_warn(&format!("Unknown provider '{}'. Valid: groq, openai, anthropic, openrouter, telegram, discord, github, brave", provider));
                        return Ok(());
                    }
                }
                self.engine.update_config(config).await?;
                print_ok(&format!("✓ {} API key saved — ready to use immediately.", provider));
                println!("  Active provider set to: {}", provider);
            }

            ConfigCommands::Model { model_name } => {
                let provider = config.ai.active_provider.clone();
                match provider {
                    crate::config::providers::ModelProvider::Anthropic =>
                        config.ai.providers.anthropic_model = Some(model_name.clone()),
                    crate::config::providers::ModelProvider::OpenAI =>
                        config.ai.providers.openai_model = Some(model_name.clone()),
                    crate::config::providers::ModelProvider::Groq =>
                        config.ai.providers.groq_model = Some(model_name.clone()),
                    crate::config::providers::ModelProvider::OpenRouter =>
                        config.ai.providers.openrouter_model = Some(model_name.clone()),
                    crate::config::providers::ModelProvider::Custom =>
                        config.ai.providers.custom_model = Some(model_name.clone()),
                }
                self.engine.update_config(config).await?;
                println!("{} Model set to: {}", Theme::success_str("✓"), model_name);
            }

            ConfigCommands::Reset => {
                let fresh = crate::config::MicrodragonConfig::default();
                self.engine.update_config(fresh).await?;
                println!("{} Configuration reset to defaults", Theme::success_str("✓"));
            }

            ConfigCommands::Export { path } => {
                let content = toml::to_string_pretty(&config)?;
                let masked = mask_api_keys(&content);
                std::fs::write(&path, masked)?;
                println!("{} Config exported to {}", Theme::success_str("✓"), path);
            }
        }
        Ok(())
    }

    async fn handle_code(&self, action: CodeCommands, _output: &str) -> Result<()> {
        match action {
            CodeCommands::Generate { description, language, output: out_file } => {
                let lang_hint = language.as_deref().unwrap_or("appropriate language");
                let prompt = format!(
                    "Generate complete, production-ready {} code for: {}\n\nRequirements:\n- Include all imports\n- Full error handling (no panic, no unwrap)\n- Inline comments explaining non-obvious logic\n- A runnable usage example at the bottom\n- No placeholder comments like TODO or 'add logic here'",
                    lang_hint, description.join(" ")
                );
                let spinner = Spinner::new(&format!("Generating {} code", lang_hint));
                let result = self.engine.process_command(&prompt).await?;
                spinner.succeed("Code ready");

                if let Some(file) = out_file {
                    // Extract code from response (strip markdown fences)
                    let code = extract_code_block(&result.response);
                    std::fs::write(&file, &code)?;
                    print_ok(&format!("Saved to {}", file));
                    markdown::render(&result.response);
                } else {
                    markdown::render(&result.response);
                }
            }

            CodeCommands::Debug { file, language } => {
                let code = std::fs::read_to_string(&file)?;
                let lang = language.as_deref().unwrap_or("auto-detect");
                let prompt = format!(
                    "Debug this {} code from '{}':\n\n```\n{}\n```\n\nFor each bug found:\n- SEVERITY: [critical/high/medium/low]\n- LINE: line number\n- BUG: what is wrong\n- FIX: the corrected code\n\nThen provide the complete fixed file.",
                    lang, file, code
                );
                let spinner = Spinner::new(&format!("Debugging {}", file));
                let result = self.engine.process_command(&prompt).await?;
                spinner.succeed("Debug complete");
                markdown::render(&result.response);
            }

            CodeCommands::Review { file } => {
                let code = std::fs::read_to_string(&file)?;
                let prompt = format!(
                    "Review this code from '{}':\n\n```\n{}\n```\n\nProvide:\n## Code Quality\n## Security Issues\n## Performance Issues\n## Improvement Suggestions\n## Overall Score (1-10)",
                    file, code
                );
                let spinner = Spinner::new(&format!("Reviewing {}", file));
                let result = self.engine.process_command(&prompt).await?;
                spinner.succeed("Review complete");
                markdown::render(&result.response);
            }

            CodeCommands::Test { path } => {
                let code = if let Some(p) = &path {
                    std::fs::read_to_string(p)?
                } else {
                    "the current project".to_string()
                };
                let prompt = format!(
                    "Write comprehensive tests for:\n\n{}\n\nInclude: unit tests, edge cases, error cases. Make them immediately runnable.",
                    code
                );
                let spinner = Spinner::new("Writing tests");
                let result = self.engine.process_command(&prompt).await?;
                spinner.succeed("Tests written");
                markdown::render(&result.response);
            }

            CodeCommands::Git { action, args } => {
                let prompt = format!(
                    "Git operation: {} {}\n\nProvide:\n1. What this command does\n2. The exact command(s) to run\n3. Any important warnings\n4. Undo command if relevant",
                    action, args.join(" ")
                );
                let result = self.engine.process_command(&prompt).await?;
                markdown::render(&result.response);
            }
        }
        Ok(())
    }

    async fn handle_research(&self, query: &str, sources: u32, output_file: Option<String>) -> Result<()> {
        let spinner = Spinner::new(&format!("Researching: {}", &query[..query.len().min(40)]));

        let prompt = format!(
            "Research this topic thoroughly: {}\n\nStructure your response exactly as:\n\n# Executive Summary\n(2-3 sentences)\n\n## Key Findings\n(numbered, most important first)\n\n## Detailed Analysis\n(deep dive into {} key areas)\n\n## Sources & References\n(cite specific sources, papers, or experts)\n\n## Conclusion\n(what to do with this information)",
            query, sources
        );
        let result = self.engine.process_command(&prompt).await?;
        spinner.succeed("Research complete");

        if let Some(file) = output_file {
            std::fs::write(&file, &result.response)?;
            print_ok(&format!("Research saved to {}", file));
        }
        markdown::render(&result.response);
        Ok(())
    }

    async fn handle_tasks(&self, action: TaskCommands) -> Result<()> {
        match action {
            TaskCommands::History { limit } => {
                let memory = self.engine.memory.read().await;
                let history = memory.get_recent_context(limit as usize).await.unwrap_or_default();
                println!("\n{} (last {} exchanges)", Theme::bold_str("Conversation History"), limit);
                println!("{}", "─".repeat(60));
                for msg in &history {
                    let role = match msg.role {
                        crate::config::providers::MessageRole::User => Theme::success_str("You"),
                        crate::config::providers::MessageRole::Assistant => Theme::info_str("MICRODRAGON"),
                        _ => Theme::muted_str("System"),
                    };
                    let preview = &msg.content[..msg.content.len().min(200)];
                    println!("\n{}: {}", role, preview);
                    if msg.content.len() > 200 { println!("  ..."); }
                }
                println!();
            }

            TaskCommands::List { status: _ } => {
                println!("\n{}", Theme::bold_str("Active Tasks"));
                let tasks = self.engine.active_tasks.iter();
                let mut count = 0;
                for entry in tasks {
                    count += 1;
                    let t = entry.value();
                    println!("  [{}] {:?} - {}", &t.id[..8], t.status, &t.input[..t.input.len().min(50)]);
                }
                if count == 0 {
                    println!("  No active tasks");
                }
            }

            _ => {
                println!("{} Command not yet implemented", "ℹ".blue());
            }
        }
        Ok(())
    }

    async fn handle_social(&self, action: SocialCommands) -> Result<()> {
        match action {
            SocialCommands::Whatsapp { action } => match action {
                WhatsappCommands::Start => {
                    println!("{} Starting WhatsApp bridge...", Theme::info_str("▸"));
                    println!("Run the Node.js bridge: cd modules/social/node_bridge && node whatsapp_bridge.js");
                }
                WhatsappCommands::Qr => {
                    println!("Scan the QR code displayed in the WhatsApp bridge console.");
                }
                _ => println!("{} Use the WhatsApp bridge console for this action", "ℹ".blue()),
            },
            SocialCommands::Telegram { action } => match action {
                TelegramCommands::Start => {
                    let config = self.engine.get_config().await;
                    if config.social.telegram_bot_token.is_none() {
                        eprintln!("{} Set Telegram bot token: microdragon config set-key telegram <token>", Theme::error_str("✗"));
                    } else {
                        println!("{} Telegram bot listening... (run telegram_bot.py)", Theme::info_str("▸"));
                    }
                }
                _ => {}
            },
            SocialCommands::Send { platform, recipient, message } => {
                let text = message.join(" ");
                let prompt = format!("Draft and send a message to {} on {}: {}", recipient, platform, text);
                let result = self.engine.process_command(&prompt).await?;
                println!("\n{}", result.response);
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_business(&self, action: BusinessCommands) -> Result<()> {
        match action {
            BusinessCommands::Market { symbol, interval } => {
                let spinner = Spinner::new(&format!("Pulling market data for {}", symbol));
                let prompt = format!(
                    "Market analysis for {} (interval: {}).\n\nStructure as:\n\n## {} Market Analysis\n\n### Price Action\n(current trend, key levels)\n\n### Technical Indicators\n- RSI:\n- MACD:\n- Bollinger Bands:\n\n### Support & Resistance\n(specific price levels)\n\n### Sentiment\n(market sentiment + news impact)\n\n### Signal\n**DIRECTION**: [LONG/SHORT/NEUTRAL]\n**Confidence**: [%]\n**Entry**: [price]\n**Stop Loss**: [price]\n**Target**: [price]\n\n> Not financial advice. Past performance ≠ future results.",
                    symbol, interval, symbol
                );
                let result = self.engine.process_command(&prompt).await?;
                spinner.succeed("Analysis complete");
                markdown::render(&result.response);
            }
            BusinessCommands::Portfolio => {
                let spinner = Spinner::new("Analysing portfolio");
                let prompt = "Analyse my investment portfolio.\n\n## Portfolio Analysis\n\n### Diversification Score\n### Risk Assessment\n### Sector Exposure\n### Rebalancing Recommendations\n### Top 3 Actions to Take Now";
                let result = self.engine.process_command(prompt).await?;
                spinner.succeed("Done");
                markdown::render(&result.response);
            }
            BusinessCommands::Risk { symbol } => {
                let spinner = Spinner::new(&format!("Risk analysis for {}", symbol));
                let prompt = format!(
                    "Risk analysis for {}.\n\n## Risk Report: {}\n\n### Volatility\n### Beta\n### Max Drawdown\n### Risk-Adjusted Return (Sharpe)\n### Risk Rating: [LOW/MEDIUM/HIGH/EXTREME]",
                    symbol, symbol
                );
                let result = self.engine.process_command(&prompt).await?;
                spinner.succeed("Done");
                markdown::render(&result.response);
            }
        }
        Ok(())
    }

    async fn handle_automate(&self, action: AutomateCommands) -> Result<()> {
        match action {
            AutomateCommands::Browser { task, url, headless } => {
                let spinner = Spinner::new("Building browser automation script");
                let prompt = format!(
                    "Write a complete Playwright (Python) automation script to: {}\n{}\nHeadless: {}\n\nRequirements:\n- Import playwright.sync_api\n- Full error handling with try/except\n- Screenshots on failure\n- Print status at each step\n- Complete, immediately runnable code",
                    task.join(" "),
                    url.map_or_else(String::new, |u| format!("Starting URL: {}", u)),
                    headless
                );
                let result = self.engine.process_command(&prompt).await?;
                spinner.succeed("Script ready");
                markdown::render(&result.response);
            }
            AutomateCommands::Desktop { task } => {
                let spinner = Spinner::new("Building desktop automation script");
                let prompt = format!(
                    "Write a complete PyAutoGUI script to: {}\n\nRequirements:\n- Full imports\n- Safety delay (pyautogui.PAUSE = 0.5)\n- try/except with cleanup\n- Print progress at each step\n- Failsafe enabled (pyautogui.FAILSAFE = True)",
                    task.join(" ")
                );
                let result = self.engine.process_command(&prompt).await?;
                spinner.succeed("Script ready");
                markdown::render(&result.response);
            }
            _ => {
                markdown::render("Use the Python automation module for this feature:\n```bash\ncd modules/automation && python run.py\n```");
            }
        }
        Ok(())
    }

    // ─── Document handler ─────────────────────────────────────────────────────

    async fn handle_document(&self, action: DocumentCommands) -> Result<()> {
        use crate::tools::modules::DocumentModule;
        use crate::tools::filesystem;

        match action {
            DocumentCommands::Word { description, output } => {
                let task = description.join(" ");
                let outpath = output.unwrap_or_else(|| {
                    filesystem::desktop_dir()
                        .map(|p| p.join("microdragon_document.docx").to_string_lossy().to_string())
                        .unwrap_or_else(|_| "document.docx".to_string())
                });
                let spinner = Spinner::new("Generating Word document content…");
                let prompt = format!(
                    "Write the full content for a Word document titled: {}

                     Use ## for section headings, - for bullets, **bold** for key terms.
                     Be comprehensive and professional. Output the raw text content only.",
                    task
                );
                let result = self.engine.process_command(&prompt).await?;
                spinner.succeed("Content ready — creating .docx file…");
                let doc_result = DocumentModule::create_docx(&task, &result.response, &outpath).await?;
                if doc_result.success {
                    print_ok(&doc_result.stdout);
                    // Open the file
                    let _ = crate::tools::modules::AppLauncher::open_file(&outpath).await;
                } else {
                    println!("  Install deps: microdragon document install");
                }
            }

            DocumentCommands::Excel { description, output } => {
                let task = description.join(" ");
                let outpath = output.unwrap_or_else(|| {
                    filesystem::desktop_dir()
                        .map(|p| p.join("microdragon_spreadsheet.xlsx").to_string_lossy().to_string())
                        .unwrap_or_else(|_| "spreadsheet.xlsx".to_string())
                });
                let spinner = Spinner::new("Generating spreadsheet data…");
                let prompt = format!(
                    "Generate spreadsheet data for: {}

                     Format as: first line = headers separated by |, then data rows separated by |
                     Example:
Name|Amount|Date|Status
Item 1|100|2026-01-01|Paid
                     Make it realistic and comprehensive with at least 10 rows.",
                    task
                );
                let result = self.engine.process_command(&prompt).await?;
                spinner.succeed("Data ready — creating .xlsx file…");
                let doc_result = DocumentModule::create_xlsx(&task, &result.response, &outpath).await?;
                if doc_result.success {
                    print_ok(&doc_result.stdout);
                    let _ = crate::tools::modules::AppLauncher::open_file(&outpath).await;
                } else {
                    println!("  Install deps: microdragon document install");
                }
            }

            DocumentCommands::Pdf { description, output } => {
                let task = description.join(" ");
                let outpath = output.unwrap_or_else(|| {
                    filesystem::desktop_dir()
                        .map(|p| p.join("microdragon_report.pdf").to_string_lossy().to_string())
                        .unwrap_or_else(|_| "report.pdf".to_string())
                });
                let spinner = Spinner::new("Generating PDF content…");
                let result = self.engine.process_command(&format!(
                    "Write a comprehensive, professional report on: {}
                     Use # for title, ## for sections, - for bullets. Be thorough.", task
                )).await?;
                spinner.succeed("Content ready — creating PDF…");
                let doc_result = DocumentModule::create_pdf(&task, &result.response, &outpath).await?;
                if doc_result.success {
                    print_ok(&doc_result.stdout);
                    let _ = crate::tools::modules::AppLauncher::open_file(&outpath).await;
                } else {
                    println!("  Install deps: microdragon document install");
                }
            }

            DocumentCommands::Pptx { description, output, slides } => {
                let task = description.join(" ");
                let outpath = output.unwrap_or_else(|| {
                    filesystem::desktop_dir()
                        .map(|p| p.join("microdragon_presentation.pptx").to_string_lossy().to_string())
                        .unwrap_or_else(|_| "presentation.pptx".to_string())
                });
                let spinner = Spinner::new("Generating presentation…");
                let result = self.engine.process_command(&format!(
                    "Create a {}-slide presentation on: {}

                     Separate each slide with two blank lines.
                     Each slide: first line = slide title, then bullet points starting with -
                     Make slides concise and impactful.",
                    slides, task
                )).await?;
                spinner.succeed("Content ready — creating .pptx file…");
                let doc_result = DocumentModule::create_pptx(&task, &result.response, &outpath).await?;
                if doc_result.success {
                    print_ok(&doc_result.stdout);
                    let _ = crate::tools::modules::AppLauncher::open_file(&outpath).await;
                } else {
                    println!("  Install deps: microdragon document install");
                }
            }

            DocumentCommands::Read { path } => {
                let spinner = Spinner::new(&format!("Reading {}…", path));
                let content = crate::tools::filesystem::read_file(&path)?;
                spinner.succeed("File read");
                let prompt = format!("Analyse and summarise this document:

{}", &content[..content.len().min(12000)]);
                let result = self.engine.process_command(&prompt).await?;
                markdown::render(&result.response);
            }

            DocumentCommands::Install => {
                let spinner = Spinner::new("Installing document dependencies…");
                let result = DocumentModule::install_deps().await?;
                spinner.succeed("Done");
                println!("{}", result.stdout);
            }
        }
        Ok(())
    }

    // ─── Design handler ───────────────────────────────────────────────────────

    async fn handle_design(&self, action: DesignCommands) -> Result<()> {
        use crate::tools::modules::{DesignModule, AppLauncher};
        use crate::tools::filesystem;

        match action {
            DesignCommands::Html { description, output, style } => {
                let task = description.join(" ");
                let outpath = output.unwrap_or_else(|| {
                    filesystem::desktop_dir()
                        .map(|p| p.join("microdragon_design.html").to_string_lossy().to_string())
                        .unwrap_or_else(|_| "design.html".to_string())
                });
                let spinner = Spinner::new("Generating HTML design…");
                let result = self.engine.process_command(&format!(
                    "Generate a complete, beautiful HTML page with embedded CSS for: {}
                     Style: {}
                     Requirements:
                     - Single file with all CSS inline in <style> tag
                     - Responsive, mobile-first design
                     - Professional and visually impressive
                     - Use CSS variables, flexbox/grid
                     - Output ONLY the complete HTML code, no explanation",
                    task, style
                )).await?;
                spinner.succeed("Design generated");
                let html = extract_code_block(&result.response);
                let write_result = DesignModule::create_html(&html, &outpath).await?;
                print_ok(&write_result.stdout);
                let _ = AppLauncher::open_file(&outpath).await;
            }

            DesignCommands::Svg { description, output } => {
                let task = description.join(" ");
                let outpath = output.unwrap_or_else(|| {
                    filesystem::desktop_dir()
                        .map(|p| p.join("microdragon_graphic.svg").to_string_lossy().to_string())
                        .unwrap_or_else(|_| "graphic.svg".to_string())
                });
                let spinner = Spinner::new("Generating SVG graphic…");
                let result = self.engine.process_command(&format!(
                    "Generate a clean, beautiful SVG graphic for: {}
                     Requirements:
                     - Valid SVG with viewBox
                     - Use paths, shapes, gradients as appropriate
                     - Professional and visually impressive design
                     - Output ONLY the SVG code, no explanation",
                    task
                )).await?;
                spinner.succeed("SVG generated");
                let svg = extract_code_block(&result.response);
                let write_result = DesignModule::create_svg(&svg, &outpath).await?;
                print_ok(&write_result.stdout);
                let _ = AppLauncher::open_file(&outpath).await;
            }

            DesignCommands::Image { description, output, width, height } => {
                let task = description.join(" ");
                let outpath = output.unwrap_or_else(|| {
                    filesystem::desktop_dir()
                        .map(|p| p.join("microdragon_image.png").to_string_lossy().to_string())
                        .unwrap_or_else(|_| "image.png".to_string())
                });
                let spinner = Spinner::new("Creating image with Pillow…");
                let result = DesignModule::create_image(&task, &outpath, width, height).await?;
                spinner.succeed("Done");
                if result.success {
                    print_ok(&format!("Image created: {}", outpath));
                    let _ = AppLauncher::open_file(&outpath).await;
                } else {
                    println!("  {}", result.stderr);
                    println!("  Install: pip install Pillow");
                }
            }

            DesignCommands::Open { path } => {
                AppLauncher::open_file(&path).await?;
                print_ok(&format!("Opened: {}", path));
            }
        }
        Ok(())
    }

    // ─── Files handler ────────────────────────────────────────────────────────

    async fn handle_files(&self, action: FilesCommands) -> Result<()> {
        use crate::tools::filesystem;

        match action {
            FilesCommands::Ls { path } => {
                let dir = path.unwrap_or_else(|| {
                    filesystem::current_dir()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_else(|_| ".".to_string())
                });
                let entries = filesystem::list_dir(&dir, 100)?;
                let display = filesystem::format_dir_listing(&dir, &entries);
                println!("{}", display);
            }

            FilesCommands::Read { path } => {
                let content = filesystem::read_file(&path)?;
                if content.len() < 3000 {
                    println!("{}", content);
                } else {
                    // AI summarises large files
                    let spinner = Spinner::new("Analysing file with AI…");
                    let result = self.engine.process_command(&format!(
                        "Analyse this file '{}' and provide a structured summary:

{}",
                        path, &content[..content.len().min(10000)]
                    )).await?;
                    spinner.succeed("Done");
                    markdown::render(&result.response);
                }
            }

            FilesCommands::Write { path, content } => {
                let text = if let Some(c) = content {
                    c
                } else {
                    println!("  Enter content (press Ctrl+D or Ctrl+Z when done):");
                    let mut buf = String::new();
                    { use std::io::Read; std::io::stdin().read_to_string(&mut buf)?; }
                    buf
                };
                filesystem::write_file(&path, &text)?;
                print_ok(&format!("Written: {} ({:.1}KB)", path, text.len() as f64 / 1024.0));
            }

            FilesCommands::Mkdir { path } => {
                filesystem::create_dir(&path)?;
                print_ok(&format!("Created directory: {}", path));
            }

            FilesCommands::Find { root, pattern } => {
                let root_dir = root.unwrap_or_else(|| ".".to_string());
                let spinner = Spinner::new(&format!("Searching for '{}' in {}…", pattern, root_dir));
                let results = filesystem::find_files(&root_dir, &pattern, 50)?;
                spinner.succeed(&format!("Found {} file(s)", results.len()));
                for r in &results {
                    println!("  {}", r);
                }
            }

            FilesCommands::Overview => {
                let overview = filesystem::pc_overview();
                println!("
{}
", overview);
                // List desktop and home
                if let Ok(desktop) = filesystem::desktop_dir() {
                    if let Ok(entries) = filesystem::list_dir(&desktop.to_string_lossy(), 20) {
                        println!("{}", filesystem::format_dir_listing("Desktop", &entries));
                    }
                }
            }

            FilesCommands::Cd { path } => {
                let entries = filesystem::list_dir(&path, 60)?;
                let display = filesystem::format_dir_listing(&path, &entries);
                println!("{}", display);
            }
        }
        Ok(())
    }

    // ─── Voice handler ────────────────────────────────────────────────────────

    async fn handle_voice(&self, action: VoiceCommands) -> Result<()> {
        use crate::tools::modules::VoiceModule;

        let config = self.engine.get_config().await;
        let provider = config.ai.active_provider.to_string();

        match action {
            VoiceCommands::Say { text } => {
                let speech = text.join(" ");
                let spinner = Spinner::new("Speaking…");
                let result = VoiceModule::speak(&speech, &provider).await?;
                spinner.succeed("Done");
                if !result.success {
                    println!("  {}", result.stderr);
                    println!("  Install: microdragon voice install");
                }
            }

            VoiceCommands::Listen { duration } => {
                let spinner = Spinner::new(&format!("Listening for {}s…", duration));
                let result = VoiceModule::listen(duration, &provider).await?;
                spinner.succeed("Transcribed");
                if result.success && !result.stdout.trim().is_empty() {
                    println!("
  You said: "{}"", result.stdout.trim());
                    // Send to AI
                    let ai_result = self.engine.process_command(result.stdout.trim()).await?;
                    markdown::render(&ai_result.response);
                    // Speak response back
                    let _ = VoiceModule::speak(&ai_result.response, &provider).await;
                } else {
                    println!("  No speech detected. Install: microdragon voice install");
                }
            }

            VoiceCommands::Setup => {
                let result = VoiceModule::setup_info(&provider).await?;
                println!("{}", result.stdout);
            }

            VoiceCommands::Install => {
                let spinner = Spinner::new("Installing voice dependencies…");
                let result = VoiceModule::install_deps().await?;
                spinner.succeed("Done");
                println!("{}", result.stdout);
            }
        }
        Ok(())
    }

    // ─── Game handler ─────────────────────────────────────────────────────────

    async fn handle_game(&self, action: GameCommands) -> Result<()> {
        use crate::tools::modules::GamingModule;

        match action {
            GameCommands::Play { game, duration } => {
                let game_name = game.join(" ");
                println!();
                println!("  🎮 MICRODRAGON Game Engine");
                println!("  ─────────────────────────────────────");
                println!("  Game:     {}", game_name);
                println!("  Duration: {}s ({:.0} minutes)", duration, duration as f64 / 60.0);
                println!("  Strategy: AI-controlled (80%+ accuracy target)");
                println!();
                println!("  ⚠  Make sure {} is open and in focus!", game_name);
                println!("  Starting in 5 seconds… switch to your game now!");
                println!();

                let result = GamingModule::play(&game_name, duration).await?;
                println!("{}", result.stdout);
            }

            GameCommands::Stop => {
                let result = GamingModule::stop().await?;
                println!("{}", result.stdout);
            }

            GameCommands::Status => {
                let result = GamingModule::status().await?;
                println!("{}", result.stdout);
            }

            GameCommands::Install => {
                let spinner = Spinner::new("Installing gaming dependencies…");
                let result = GamingModule::install_deps().await?;
                spinner.succeed("Done");
                println!("{}", result.stdout);
                println!("  Optional (Windows only): pip install vgamepad");
            }
        }
        Ok(())
    }

    // ─── GitHub handler ───────────────────────────────────────────────────────

    async fn handle_github(&self, action: GithubCommands) -> Result<()> {
        use crate::tools::modules::GitHubModule;

        match action {
            GithubCommands::Review { pr_url, post } => {
                let spinner = Spinner::new("Fetching PR and running AI review…");
                let result = GitHubModule::review_pr(&pr_url, post).await?;
                spinner.succeed("Review complete");
                markdown::render(&result.stdout);
            }

            GithubCommands::Repo { repo } => {
                let parts: Vec<&str> = repo.splitn(2, '/').collect();
                if parts.len() != 2 {
                    print_warn("Format: microdragon github repo owner/repo-name");
                    return Ok(());
                }
                let spinner = Spinner::new("Fetching repo stats…");
                let result = GitHubModule::repo_overview(parts[0], parts[1]).await?;
                spinner.succeed("Done");
                markdown::render(&result.stdout);
            }

            GithubCommands::Issue { repo, title } => {
                let parts: Vec<&str> = repo.splitn(2, '/').collect();
                if parts.len() != 2 {
                    print_warn("Format: microdragon github issue owner/repo \"Title of issue\"");
                    return Ok(());
                }
                let issue_title = title.join(" ");
                let spinner = Spinner::new("Generating issue content…");
                let result = self.engine.process_command(&format!(
                    "Write a GitHub issue for the repo {}/{}.
                     Title: {}

                     Write a proper issue body with:
                     ## Description
## Steps to Reproduce
## Expected Behavior
## Actual Behavior",
                    parts[0], parts[1], issue_title
                )).await?;
                spinner.succeed("Creating issue…");
                let create_result = GitHubModule::create_issue(
                    parts[0], parts[1], &issue_title, &result.response
                ).await?;
                println!("{}", create_result.stdout);
            }
        }
        Ok(())
    }

    // ─── Open handler ─────────────────────────────────────────────────────────

    async fn handle_open(&self, target: &str) -> Result<()> {
        use crate::tools::modules::AppLauncher;
        let result = AppLauncher::open_file(target).await?;
        if result.success {
            print_ok(&format!("Opened: {}", target));
        } else {
            let result2 = AppLauncher::open(target).await?;
            if result2.success {
                print_ok(&format!("Opened: {}", target));
            } else {
                print_warn(&format!("Could not open '{}': {}", target, result.stderr));
            }
        }
        Ok(())
    }

    /// Called when a command is run but no API key is configured.
    /// Instead of dying silently, offer to set it up right now.
    async fn offer_inline_setup(&self, _context: &str) -> Result<()> {
        println!();
        print_warn("No API key configured.");
        println!();
        println!("  You can configure MICRODRAGON right now without restarting.");
        println!();
        println!("  Option A — run the full setup wizard:");
        println!("    microdragon setup");
        println!();
        println!("  Option B — set a key instantly:");
        println!("    microdragon config set-key groq gsk_...");
        println!("    microdragon config set-key openai sk-...");
        println!("    microdragon config set-key anthropic sk-ant-...");
        println!();

        // Offer inline key entry right now
        print!("  Enter your API key now (or press Enter to skip): ");
        let _ = std::io::Write::flush(&mut std::io::stdout());
        let mut key = String::new();
        { use std::io::BufRead; std::io::BufReader::new(std::io::stdin()).read_line(&mut key)?; }
        let key = key.trim().to_string();

        if !key.is_empty() {
            // Auto-detect provider from key prefix
            let mut config = self.engine.get_config().await;
            let detected_provider = if key.starts_with("gsk_") {
                config.ai.active_provider = crate::config::providers::ModelProvider::Groq;
                config.ai.providers.groq_api_key = Some(key.clone());
                "Groq"
            } else if key.starts_with("sk-ant-") {
                config.ai.active_provider = crate::config::providers::ModelProvider::Anthropic;
                config.ai.providers.anthropic_api_key = Some(key.clone());
                "Anthropic"
            } else if key.starts_with("sk-or-") {
                config.ai.active_provider = crate::config::providers::ModelProvider::OpenRouter;
                config.ai.providers.openrouter_api_key = Some(key.clone());
                "OpenRouter"
            } else if key.starts_with("sk-") {
                config.ai.active_provider = crate::config::providers::ModelProvider::OpenAI;
                config.ai.providers.openai_api_key = Some(key.clone());
                "OpenAI"
            } else {
                // Unknown prefix — ask which provider
                println!("  Key prefix not recognised. Which provider?");
                println!("  1) Groq  2) OpenAI  3) Anthropic  4) OpenRouter  5) Custom");
                print!("  Choice [1-5]: ");
                let _ = std::io::Write::flush(&mut std::io::stdout());
                let mut choice = String::new();
                { use std::io::BufRead; std::io::BufReader::new(std::io::stdin()).read_line(&mut choice)?; }
                match choice.trim() {
                    "2" => {
                        config.ai.active_provider = crate::config::providers::ModelProvider::OpenAI;
                        config.ai.providers.openai_api_key = Some(key.clone());
                        "OpenAI"
                    }
                    "3" => {
                        config.ai.active_provider = crate::config::providers::ModelProvider::Anthropic;
                        config.ai.providers.anthropic_api_key = Some(key.clone());
                        "Anthropic"
                    }
                    "4" => {
                        config.ai.active_provider = crate::config::providers::ModelProvider::OpenRouter;
                        config.ai.providers.openrouter_api_key = Some(key.clone());
                        "OpenRouter"
                    }
                    "5" => {
                        print!("  Enter endpoint URL: ");
                        let _ = std::io::Write::flush(&mut std::io::stdout());
                        let mut ep = String::new();
                        { use std::io::BufRead; std::io::BufReader::new(std::io::stdin()).read_line(&mut ep)?; }
                        config.ai.providers.custom_endpoint = Some(ep.trim().to_string());
                        config.ai.active_provider = crate::config::providers::ModelProvider::Custom;
                        "Custom"
                    }
                    _ => {
                        config.ai.active_provider = crate::config::providers::ModelProvider::Groq;
                        config.ai.providers.groq_api_key = Some(key.clone());
                        "Groq"
                    }
                }
            };

            self.engine.update_config(config).await?;
            print_ok(&format!("✓ {} API key saved — you're ready.", detected_provider));
            println!("  Re-run your command now.");
        } else {
            println!("  Skipped. Run 'microdragon setup' when ready.");
        }

        println!();
        Ok(())
    }

    async fn clear_history(&self) -> Result<()> {
        let mut memory = self.engine.memory.write().await;
        memory.clear_context().await?;
        println!("{} Conversation history cleared", Theme::success_str("✓"));
        Ok(())
    }

    fn show_version(&self) {
        println!("MICRODRAGON Universal AI Agent");
        println!("Version: 0.1.0");
        println!("Core: Rust (tokio async)");
        println!("Modules: Python + Node.js");
        println!("License: MIT");
    }
}

fn mask_api_keys(content: &str) -> String {
    let re = regex::Regex::new(r#"(api_key\s*=\s*")([^"]{8})[^"]*(")"#).unwrap();
    re.replace_all(content, |caps: &regex::Captures| {
        format!("{}{}...MASKED{}", &caps[1], &caps[2], &caps[3])
    }).to_string()
}

/// Extract the first code block from an AI response (strips ``` fences)
fn extract_code_block(text: &str) -> String {
    let mut in_block = false;
    let mut lines: Vec<&str> = Vec::new();
    for line in text.lines() {
        if line.trim_start().starts_with("```") {
            if in_block { break; }
            in_block = true;
            continue;
        }
        if in_block {
            lines.push(line);
        }
    }
    if lines.is_empty() { text.to_string() } else { lines.join("\n") }
}
