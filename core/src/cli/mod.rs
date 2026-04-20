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
pub use simple_mode::{SimpleMode, GoalFlow};

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::sync::Arc;

use crate::engine::MicrodragonEngine;
use self::commands::*;
use self::interactive::InteractiveMode;
use self::setup::SetupWizard;
use self::display::{print_ok, print_err, print_warn, print_info, Spinner, response_footer};
use self::stream_renderer::print_response;
use self::theme::Theme;

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

    /// Show version information
    #[command(alias = "v")]
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
    pub fn new(engine: MicrodragonEngine) -> Self {
        Self {
            engine: Arc::new(engine),
        }
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
        }
    }

    async fn run_ask(&self, input: &str, output_format: &str) -> Result<()> {
        let config = self.engine.get_config().await;
        if !config.is_configured() {
            print_warn("MICRODRAGON is not configured. Run 'microdragon setup' first.");
            return Ok(());
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
                print_response(&result.response, true).await;
                response_footer(&result.provider, &result.model, result.tokens_used, result.latency_ms);
            }
        }
        Ok(())
    }

    async fn run_streaming(&self, input: &str) -> Result<()> {
        let config = self.engine.get_config().await;
        if !config.is_configured() {
            print_warn("MICRODRAGON is not configured. Run 'microdragon setup' first.");
            return Ok(());
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
                match provider.to_lowercase().as_str() {
                    "anthropic" => config.ai.providers.anthropic_api_key = Some(key),
                    "openai" => config.ai.providers.openai_api_key = Some(key),
                    "groq" => config.ai.providers.groq_api_key = Some(key),
                    "openrouter" => config.ai.providers.openrouter_api_key = Some(key),
                    _ => { eprintln!("{} Unknown provider", Theme::error_str("✗")); return Ok(()); }
                }
                self.engine.update_config(config).await?;
                println!("{} API key saved for {}", Theme::success_str("✓"), provider);
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

    async fn handle_code(&self, action: CodeCommands, output: &str) -> Result<()> {
        match action {
            CodeCommands::Generate { description, language, output: out_file } => {
                let lang_hint = language.as_deref().unwrap_or("appropriate language");
                let prompt = format!(
                    "Write complete, production-ready {} code for: {}\n\nInclude: imports, error handling, comments, and a usage example.",
                    lang_hint,
                    description.join(" ")
                );
                let result = self.engine.process_command(&prompt).await?;
                if let Some(file) = out_file {
                    std::fs::write(&file, &result.response)?;
                    println!("{} Code written to {}", Theme::success_str("✓"), file);
                } else {
                    println!("\n{}", result.response);
                }
            }

            CodeCommands::Debug { file, language } => {
                let code = std::fs::read_to_string(&file)?;
                let lang = language.as_deref().unwrap_or("auto-detect");
                let prompt = format!(
                    "Debug this {} code from file '{}':\n\n```\n{}\n```\n\nIdentify all bugs, explain each issue, and provide a fixed version.",
                    lang, file, code
                );
                let result = self.engine.process_command(&prompt).await?;
                println!("\n{}", result.response);
            }

            CodeCommands::Review { file } => {
                let code = std::fs::read_to_string(&file)?;
                let prompt = format!(
                    "Review this code from '{}':\n\n```\n{}\n```\n\nProvide: code quality assessment, security issues, performance issues, and improvement suggestions.",
                    file, code
                );
                let result = self.engine.process_command(&prompt).await?;
                println!("\n{}", result.response);
            }

            CodeCommands::Test { path } => {
                let code = if let Some(p) = path {
                    std::fs::read_to_string(&p)?
                } else {
                    "the current project".to_string()
                };
                let prompt = format!("Write comprehensive tests for:\n\n{}", code);
                let result = self.engine.process_command(&prompt).await?;
                println!("\n{}", result.response);
            }

            CodeCommands::Git { action, args } => {
                let prompt = format!(
                    "Help me with this git operation: {} {}\nExplain what it does and suggest the exact command(s).",
                    action, args.join(" ")
                );
                let result = self.engine.process_command(&prompt).await?;
                println!("\n{}", result.response);
            }
        }
        Ok(())
    }

    async fn handle_research(&self, query: &str, sources: u32, output_file: Option<String>) -> Result<()> {
        let spinner = Spinner::new(&format!("Researching: {}", &query[..query.len().min(40)]));

        let prompt = format!(
            "Research this topic thoroughly: {}\n\nGather from {} key sources. Structure your response with:\n1. Executive Summary\n2. Key Findings\n3. Detailed Analysis\n4. Sources/References\n5. Conclusions",
            query, sources
        );
        let result = self.engine.process_command(&prompt).await?;
        spinner.succeed("Research complete");

        if let Some(file) = output_file {
            std::fs::write(&file, &result.response)?;
            print_ok(&format!("Research saved to {}", file));
        } else {
            print_response(&result.response, false).await;
        }
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

            TaskCommands::List { status } => {
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
                let prompt = format!(
                    "Provide a comprehensive market analysis for {} with interval {}. Include: current price trends, technical indicators, support/resistance levels, sentiment analysis, and trading recommendations.",
                    symbol, interval
                );
                let result = self.engine.process_command(&prompt).await?;
                println!("\n{}", result.response);
            }
            BusinessCommands::Portfolio => {
                let prompt = "Review my investment portfolio. Provide diversification analysis, risk assessment, and rebalancing suggestions.";
                let result = self.engine.process_command(prompt).await?;
                println!("\n{}", result.response);
            }
            BusinessCommands::Risk { symbol } => {
                let prompt = format!("Analyze risk factors for {} including volatility, beta, drawdown history, and risk-adjusted return metrics.", symbol);
                let result = self.engine.process_command(&prompt).await?;
                println!("\n{}", result.response);
            }
        }
        Ok(())
    }

    async fn handle_automate(&self, action: AutomateCommands) -> Result<()> {
        match action {
            AutomateCommands::Browser { task, url, headless } => {
                let prompt = format!(
                    "Create a Playwright automation script to: {}\n{}\nHeadless: {}\n\nOutput a complete, working Python script.",
                    task.join(" "),
                    url.map_or_else(|| String::new(), |u| format!("Starting URL: {}", u)),
                    headless
                );
                let result = self.engine.process_command(&prompt).await?;
                println!("\n{}", result.response);
            }
            AutomateCommands::Desktop { task } => {
                let prompt = format!(
                    "Create a PyAutoGUI script to automate this desktop task: {}\n\nOutput a complete, safe Python script with error handling.",
                    task.join(" ")
                );
                let result = self.engine.process_command(&prompt).await?;
                println!("\n{}", result.response);
            }
            _ => {
                println!("{} Automation feature - use the Python automation module", "ℹ".blue());
            }
        }
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
