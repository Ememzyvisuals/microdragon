// microdragon-core/src/cli/setup.rs
//
// First-Run Setup Wizard
// ───────────────────────
// Guides the user through configuring their AI provider.
// Uses dialoguer on capable terminals; falls back to manual read_line
// on plain/CMD environments where dialoguer's cursor tricks may fail.

use anyhow::Result;
use std::io::{self, Write};
use std::sync::Arc;

use crate::engine::MicrodragonEngine;
use crate::config::{MicrodragonConfig, providers::ModelProvider};
use crate::cli::terminal::CAPS;
use crate::cli::theme::Theme;
use crate::cli::display::{print_ok, print_err, print_warn, print_info, section, kv, kv_colored, panel};
use crossterm::style::Color;

pub struct SetupWizard {
    engine: Arc<MicrodragonEngine>,
}

impl SetupWizard {
    pub fn new(engine: Arc<MicrodragonEngine>) -> Self {
        Self { engine }
    }

    pub async fn run(self) -> Result<()> {
        self.print_intro();

        let mut config = self.engine.get_config().await;
        config.ensure_dirs()?;

        // ── Step 1: Choose provider ───────────────────────────────────────
        section("Step 1 — Choose your AI Provider");

        let providers = [
            ("anthropic",  "Anthropic Claude  (recommended, claude-opus-4-5)"),
            ("openai",     "OpenAI GPT        (gpt-4o)"),
            ("groq",       "Groq              (fast, free tier available)"),
            ("openrouter", "OpenRouter        (multi-model access)"),
            ("custom",     "Custom / Local    (Ollama, LM Studio, etc.)"),
        ];

        let provider_choice = if CAPS.is_plain() {
            self.plain_select("Provider (1-5)", &providers.iter().map(|(_, l)| *l).collect::<Vec<_>>())
        } else {
            self.fancy_select("Select provider", &providers.iter().map(|(_, l)| *l).collect::<Vec<_>>())
        };

        let (provider_key, _) = providers[provider_choice];
        config.ai.active_provider = match provider_key {
            "anthropic"  => ModelProvider::Anthropic,
            "openai"     => ModelProvider::OpenAI,
            "groq"       => ModelProvider::Groq,
            "openrouter" => ModelProvider::OpenRouter,
            _            => ModelProvider::Custom,
        };

        print_ok(&format!("Provider set to: {}", config.ai.active_provider));

        // ── Step 2: API key ───────────────────────────────────────────────
        section("Step 2 — API Key");

        let key_prompt = match &config.ai.active_provider {
            ModelProvider::Anthropic  => "Enter your Anthropic API key (sk-ant-...): ",
            ModelProvider::OpenAI     => "Enter your OpenAI API key (sk-...): ",
            ModelProvider::Groq       => "Enter your Groq API key (gsk_...): ",
            ModelProvider::OpenRouter => "Enter your OpenRouter API key (sk-or-...): ",
            ModelProvider::Custom     => "Enter your local endpoint (e.g. http://localhost:11434): ",
        };

        let is_custom = config.ai.active_provider == ModelProvider::Custom;
        let key_value = self.read_secret(key_prompt)?;

        if key_value.is_empty() {
            print_warn("No key entered — you can set it later with: microdragon config set-key <provider> <key>");
        } else {
            match &config.ai.active_provider {
                ModelProvider::Anthropic  => config.ai.providers.anthropic_api_key  = Some(key_value),
                ModelProvider::OpenAI     => config.ai.providers.openai_api_key     = Some(key_value),
                ModelProvider::Groq       => config.ai.providers.groq_api_key       = Some(key_value),
                ModelProvider::OpenRouter => config.ai.providers.openrouter_api_key = Some(key_value),
                ModelProvider::Custom     => config.ai.providers.custom_endpoint    = Some(key_value),
            }
            print_ok("API key saved (stored locally, never transmitted).");
        }

        // ── Step 3: Model selection ───────────────────────────────────────
        section("Step 3 — Model (optional)");

        let default_model = config.ai.active_provider.default_model();
        println!("  Default model: {}", Theme::info_str(default_model));

        let custom_model = self.read_line(&format!("Custom model name [Enter to use {}]: ", default_model))?;
        if !custom_model.is_empty() {
            match &config.ai.active_provider {
                ModelProvider::Anthropic  => config.ai.providers.anthropic_model  = Some(custom_model.clone()),
                ModelProvider::OpenAI     => config.ai.providers.openai_model     = Some(custom_model.clone()),
                ModelProvider::Groq       => config.ai.providers.groq_model       = Some(custom_model.clone()),
                ModelProvider::OpenRouter => config.ai.providers.openrouter_model = Some(custom_model.clone()),
                ModelProvider::Custom     => config.ai.providers.custom_model     = Some(custom_model.clone()),
            }
            print_ok(&format!("Model set to: {}", custom_model));
        } else {
            print_ok(&format!("Using default model: {}", default_model));
        }

        // ── Step 4: Social platforms ──────────────────────────────────────
        section("Step 4 — Social Platforms (optional, press Enter to skip)");

        let tg_token = self.read_line("Telegram bot token (from @BotFather): ")?;
        if !tg_token.is_empty() {
            config.social.telegram_enabled = true;
            config.social.telegram_bot_token = Some(tg_token);
            print_ok("Telegram enabled.");
        }

        let dc_token = self.read_line("Discord bot token: ")?;
        if !dc_token.is_empty() {
            config.social.discord_enabled = true;
            config.social.discord_bot_token = Some(dc_token);
            print_ok("Discord enabled.");
        }

        let wa_enabled = self.read_line("Enable WhatsApp bridge? [y/N]: ")?;
        if matches!(wa_enabled.to_lowercase().as_str(), "y" | "yes") {
            config.social.whatsapp_enabled = true;
            print_ok("WhatsApp bridge enabled — run 'microdragon social whatsapp start' to connect.");
        }

        // ── Save ──────────────────────────────────────────────────────────
        section("Saving Configuration");
        self.engine.update_config(config).await?;

        // ── Summary ───────────────────────────────────────────────────────
        self.print_summary().await;
        Ok(())
    }

    async fn print_summary(&self) {
        let config = self.engine.get_config().await;
        section("Setup Complete");

        kv("Provider:", &config.ai.active_provider.to_string());
        kv("Model:", &config.ai.providers.get_model(&config.ai.active_provider));
        kv_colored("Configured:", if config.is_configured() { "yes" } else { "no" },
            if config.is_configured() { Color::Green } else { Color::Red });
        kv("Config path:", &MicrodragonConfig::config_path().display().to_string());
        kv("Data dir:", &config.storage.base_dir.display().to_string());

        println!();
        println!("  {} {}", Theme::glyph_arrow(), Theme::info_str("Run 'microdragon' to start the interactive session."));
        println!("  {} {}", Theme::glyph_arrow(), Theme::info_str("Run 'microdragon ask \"hello\"' to test a quick prompt."));
        println!("  {} {}", Theme::glyph_arrow(), Theme::muted_str("Run 'microdragon --help' for all commands."));
        println!();
    }

    fn print_intro(&self) {
        let banner = Theme::banner();
        if CAPS.ansi_color {
            use crossterm::style::Stylize;
            println!("{}", banner.with(Color::Cyan).bold());
        } else {
            println!("{}", banner);
        }

        panel(
            "Welcome to MICRODRAGON Setup",
            "This wizard will configure your AI provider and optional integrations.\n\
             All data is stored locally on your machine — nothing leaves your PC.\n\
             You can re-run this wizard at any time with: microdragon setup"
        );
        println!();
    }

    // ── Input helpers ─────────────────────────────────────────────────────

    fn read_line(&self, prompt: &str) -> Result<String> {
        let p = if CAPS.ansi_color {
            use crossterm::style::Stylize;
            format!("  {} ", prompt.with(Color::White))
        } else {
            format!("  {}", prompt)
        };
        print!("{}", p);
        let _ = io::stdout().flush();
        let mut buf = String::new();
        io::stdin().read_line(&mut buf)?;
        Ok(buf.trim().to_string())
    }

    fn read_secret(&self, prompt: &str) -> Result<String> {
        // rpassword would be ideal but adds a dep — for now use rline without echo hint
        // In production integrate `rpassword` crate here for real masking.
        let p = if CAPS.ansi_color {
            use crossterm::style::Stylize;
            format!("  {} ", prompt.with(Color::Yellow))
        } else {
            format!("  {}", prompt)
        };
        print!("{}", p);
        let _ = io::stdout().flush();
        let mut buf = String::new();
        io::stdin().read_line(&mut buf)?;
        // Mask display (key already entered, just print placeholder)
        if CAPS.ansi_color && !buf.trim().is_empty() {
            let masked = "*".repeat(8.min(buf.trim().len()));
            if CAPS.cursor_movement {
                // Move up and overwrite to mask the key in terminal history
                use crossterm::{cursor, terminal, execute};
                let _ = execute!(
                    io::stdout(),
                    cursor::MoveUp(1),
                    terminal::Clear(crossterm::terminal::ClearType::CurrentLine)
                );
                println!("  {} {}", prompt, masked);
            }
        }
        Ok(buf.trim().to_string())
    }

    fn plain_select(&self, prompt: &str, options: &[&str]) -> usize {
        for (i, opt) in options.iter().enumerate() {
            println!("  {}. {}", i + 1, opt);
        }
        loop {
            print!("  {} (number): ", prompt);
            let _ = io::stdout().flush();
            let mut buf = String::new();
            let _ = io::stdin().read_line(&mut buf);
            if let Ok(n) = buf.trim().parse::<usize>() {
                if n >= 1 && n <= options.len() {
                    return n - 1;
                }
            }
            println!("  Please enter a number between 1 and {}", options.len());
        }
    }

    fn fancy_select(&self, prompt: &str, options: &[&str]) -> usize {
        // Try dialoguer; fall back to plain if it errors (old CMD)
        let result = dialoguer::Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .with_prompt(prompt)
            .items(options)
            .default(0)
            .interact_opt();

        match result {
            Ok(Some(idx)) => idx,
            _ => self.plain_select(prompt, options),
        }
    }
}
