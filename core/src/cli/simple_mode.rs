// microdragon/core/src/cli/simple_mode.rs
//
// Microdragon Simple Mode — Conversational entry point
//
// What users see on every launch (after first_launch consent):
//
//   🐉 Microdragon v0.1.0-beta  (short_hash)
//   I breathe fire on bad code and leave clean commits in the ashes.
//
//   > ▌
//
// That's it. The tagline changes every launch.
// No commands. No menus. Just the prompt.
//
// © 2026 EMEMZYVISUALS DIGITALS — Emmanuel Ariyo

use anyhow::Result;
use std::io::{self, Write};
use std::sync::Arc;
use crossterm::style::{Color, Stylize};
use crate::engine::MicrodragonEngine;
use crate::cli::theme::{Theme, random_tagline};
use crate::cli::terminal::CAPS;

// ─── Goal flows ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum GoalFlow {
    Code,
    Security,
    Research,
    CreateContent,
    BuildApp,
    AnalyseData,
    PlayGame,
    Finance,
    Write,
    Assistant,
    Train,
    General,
}

impl GoalFlow {
    pub fn from_message(msg: &str) -> Self {
        let m = msg.to_lowercase();
        if m.contains("code") || m.contains("debug") || m.contains("python")
            || m.contains("rust") || m.contains("javascript") || m.contains("bug")
            || m.contains("function") || m.contains("script") || m.contains("api")
            || m.contains("build") && (m.contains("app") || m.contains("api")) {
            return GoalFlow::Code;
        }
        if m.contains("security") || m.contains("vulnerability") || m.contains("pentest")
            || m.contains("exploit") || m.contains("audit") || m.contains("owasp")
            || m.contains("hack") || m.contains("cve") || m.contains("injection")
            || m.contains("xss") || m.contains("sqli") {
            return GoalFlow::Security;
        }
        if m.contains("game") || m.contains("play") || m.contains("gta")
            || m.contains("mortal kombat") || m.contains("counter-strike")
            || m.contains("need for speed") || m.contains("cs2") {
            return GoalFlow::PlayGame;
        }
        if m.contains("tiktok") || m.contains("youtube") || m.contains("video")
            || m.contains("content") || m.contains("reel") || m.contains("post") {
            return GoalFlow::CreateContent;
        }
        if m.contains("stock") || m.contains("crypto") || m.contains("market")
            || m.contains("signal") || m.contains("bitcoin") || m.contains("invest")
            || m.contains("aapl") || m.contains("trading") {
            return GoalFlow::Finance;
        }
        if m.contains("research") || m.contains("find") || m.contains("what is")
            || m.contains("compare") || m.contains("explain") {
            return GoalFlow::Research;
        }
        if m.contains("analyse") || m.contains("analyze") || m.contains("data")
            || m.contains("csv") || m.contains("spreadsheet") {
            return GoalFlow::AnalyseData;
        }
        if m.contains("write") || m.contains("draft") || m.contains("document")
            || m.contains("proposal") || m.contains("email") && m.contains("write") {
            return GoalFlow::Write;
        }
        if m.contains("meeting") || m.contains("schedule") || m.contains("briefing")
            || m.contains("plan my") || m.contains("remind") || m.contains("calendar") {
            return GoalFlow::Assistant;
        }
        if m.contains("fine-tune") || m.contains("train") || m.contains("finetune")
            || m.contains("model") && m.contains("train") {
            return GoalFlow::Train;
        }
        GoalFlow::General
    }

    pub fn clarifying_question(&self) -> Option<&'static str> {
        match self {
            GoalFlow::CreateContent => Some("Platform (TikTok / YouTube / Instagram) and topic?"),
            GoalFlow::PlayGame      => Some("Which game? And what do you want me to do?"),
            GoalFlow::Security      => Some("Target is a system you own or have written authorisation for?"),
            GoalFlow::Train         => Some("Provider — OpenAI, local GPU (LoRA), or Ollama (free)?"),
            GoalFlow::Assistant     => Some("Your role — developer, CEO, trader, or gamer?"),
            _                       => None,
        }
    }

    pub fn action_preview(&self) -> &'static str {
        match self {
            GoalFlow::Code          => "Reading code. Finding issues. Fixing them. Running the result.",
            GoalFlow::Security      => "Loading security engine. OWASP scanner. Threat model. CVE lookup.",
            GoalFlow::Research      => "Searching across sources. Synthesising. Giving you one clean report.",
            GoalFlow::CreateContent => "Script. Hooks. Timestamps. Hashtag pack. Full content kit.",
            GoalFlow::BuildApp      => "Architecture. Every file. Tests. README. Ready to run.",
            GoalFlow::AnalyseData   => "Loading data. Finding patterns. Key insights.",
            GoalFlow::PlayGame      => "Opening game engine. Make sure the game is open and in focus.",
            GoalFlow::Finance       => "Pulling live data. RSI. MACD. Signal with confidence score.",
            GoalFlow::Write         => "Writing the full document. Saving as a file.",
            GoalFlow::Assistant     => "Preparing your briefing.",
            GoalFlow::Train         => "Checking hardware. Building dataset. Starting fine-tune.",
            GoalFlow::General       => "On it.",
        }
    }

    pub fn followup(&self) -> &'static str {
        match self {
            GoalFlow::Code      => "Want tests written, or a full project review?",
            GoalFlow::Security  => "Want a STRIDE threat model or incident response playbook?",
            GoalFlow::Research  => "Save this as a document?",
            GoalFlow::Finance   => "Set up a background alert for this ticker?",
            GoalFlow::Write     => "Open this in Word?",
            _                   => "What's next?",
        }
    }
}

// ─── Simple Mode ──────────────────────────────────────────────────────────────

pub struct SimpleMode {
    engine: Arc<MicrodragonEngine>,
    switching_to_pro: bool,
}

impl SimpleMode {
    pub fn new(engine: Arc<MicrodragonEngine>) -> Self {
        Self { engine, switching_to_pro: false }
    }

    pub async fn run(&mut self) -> Result<()> {
        self.print_header();

        loop {
            let raw = self.prompt()?;
            let input = raw.trim();
            if input.is_empty() { continue; }

            // Built-in slash commands
            match input {
                "/help" | "help" => { self.print_help(); continue; }
                "/status" | "status" => { self.print_status().await; continue; }
                "/clear" | "clear" => { print!("\x1B[2J\x1B[1;1H"); self.print_header(); continue; }
                "/exit" | "exit" | "quit" | "q" => {
                    println!();
                    if CAPS.ansi {
                        println!("  {}", "🐉 Goodbye.".to_string().with(Theme::GREEN));
                    } else {
                        println!("  Goodbye.");
                    }
                    println!();
                    return Ok(());
                }
                _ => {}
            }

            // Classify → preview → optional clarify → execute
            let flow = GoalFlow::from_message(input);

            println!();
            if CAPS.ansi {
                println!("  {}", flow.action_preview().to_string().with(Theme::GREEN));
            } else {
                println!("  {}", flow.action_preview());
            }
            println!();

            let full_input = if let Some(q) = flow.clarifying_question() {
                if CAPS.ansi {
                    println!("  {}", q.to_string().with(Color::White));
                } else {
                    println!("  {}", q);
                }
                let clarification = self.prompt()?;
                let c = clarification.trim();
                if !c.is_empty() { format!("{} — {}", input, c) } else { input.to_string() }
            } else {
                input.to_string()
            };

            // Execute via engine
            if CAPS.ansi { print!("  {}", "thinking...".to_string().with(Theme::DIM)); }
            else         { print!("  ..."); }
            io::stdout().flush()?;

            match self.engine.process_command(&full_input).await {
                Ok(result) => {
                    print!("\r                    \r");
                    println!();
                    for line in result.response.lines() {
                        println!("  {}", line);
                    }
                    println!();
                    if CAPS.ansi {
                        println!("  {}", flow.followup().to_string().with(Theme::DIM));
                    } else {
                        println!("  {}", flow.followup());
                    }
                    println!();
                }
                Err(e) => {
                    print!("\r                    \r");
                    println!();
                    if CAPS.ansi {
                        println!("  {} {}", "✗".to_string().with(Theme::FIRE), e);
                    } else {
                        println!("  Error: {}", e);
                    }
                    println!("  Not configured? Run: microdragon setup");
                    println!();
                }
            }
        }
    }

    fn print_header(&self) {
        let tagline = random_tagline();
        let hash = option_env!("GIT_SHORT_SHA").unwrap_or("dev");
        let version = env!("CARGO_PKG_VERSION");

        println!();

        if CAPS.ansi {
            // Version line — fire red emoji + green text, exactly like OpenClaw
            println!("  {} {}",
                "🐉".to_string().with(Theme::FIRE),
                format!("Microdragon {} ({})", version, hash).with(Theme::FIRE).bold()
            );
            // Tagline in green — changes every launch
            println!("  {}", tagline.to_string().with(Theme::GREEN));
            println!();
            // The prompt is ready — no extra headers needed
        } else {
            println!("  Microdragon {} ({})", version, hash);
            println!("  {}", tagline);
            println!();
        }
    }

    fn print_help(&self) {
        println!();
        if CAPS.ansi {
            println!("  {} {}", "🐉".to_string().with(Theme::GREEN),
                     "Examples — describe your goal in plain English:".to_string().with(Theme::GREEN).bold());
        }
        let examples = [
            ("Debug my Python login function", "code"),
            ("Audit my codebase for SQL injection", "security"),
            ("Pentest this API (I own it)", "security"),
            ("Build a REST API in Go", "code"),
            ("Research the best AI agents in 2026", "research"),
            ("Analyse my sales.csv", "data"),
            ("What is the AAPL signal right now?", "finance"),
            ("Play GTA V for me", "game"),
            ("Write a TikTok script about Microdragon", "content"),
            ("Fine-tune a model on my data", "train"),
            ("Draft a client proposal", "write"),
            ("Prepare my morning briefing as a developer", "assistant"),
        ];
        println!();
        for (ex, label) in &examples {
            if CAPS.ansi {
                println!("  {}  {} {}",
                    "→".to_string().with(Theme::DIM),
                    format!("[{}]", label).to_string().with(Theme::GREEN),
                    ex
                );
            } else {
                println!("  [{}]  {}", label, ex);
            }
        }
        println!();
        if CAPS.ansi {
            println!("  {}  /status /clear /exit", "Commands:".to_string().with(Theme::DIM));
        } else {
            println!("  Commands: /status /clear /exit");
        }
        println!();
    }

    async fn print_status(&self) {
        let h = self.engine.health_check().await;
        println!();
        if CAPS.ansi {
            let status = if h.is_healthy { 
                "ready".to_string().with(Theme::GREEN)
            } else { 
                "not configured — run: microdragon setup".to_string().with(Theme::FIRE)
            };
            println!("  provider  {}", h.provider.to_string().with(Theme::GREEN));
            println!("  model     {}", h.model.to_string().with(Theme::GREEN));
            println!("  status    {}", status);
        } else {
            println!("  provider: {}", h.provider);
            println!("  model:    {}", h.model);
            println!("  status:   {}", if h.is_healthy { "ready" } else { "run: microdragon setup" });
        }
        println!();
    }

    fn prompt(&self) -> Result<String> {
        if CAPS.ansi {
            print!("  {}", "> ".to_string().with(Theme::GREEN).bold());
        } else {
            print!("  > ");
        }
        io::stdout().flush()?;
        let mut s = String::new();
        io::stdin().read_line(&mut s)?;
        Ok(s)
    }

    pub fn is_pro_mode(&self) -> bool { self.switching_to_pro }
}

// ─── Flyer / Design detection (appended patch) ────────────────────────────────
// When the user says anything about a flyer, poster, or design,
// Microdragon does NOT proceed blindly. It asks clarifying questions first,
// THEN assembles all assets, THEN runs Photoshop or the SVG renderer.

pub fn is_flyer_request(input: &str) -> bool {
    let m = input.to_lowercase();
    (m.contains("flyer") || m.contains("poster") || m.contains("banner")
        || m.contains("design") && (m.contains("birthday") || m.contains("business")
            || m.contains("event") || m.contains("invitation") || m.contains("invite")
            || m.contains("party") || m.contains("sale") || m.contains("promo")))
}

pub fn flyer_clarification_questions(input: &str) -> Vec<(&'static str, &'static str)> {
    let m = input.to_lowercase();
    if m.contains("birthday") || m.contains("party") {
        vec![
            ("Name & age",  "Whose birthday is it and how old are they turning?"),
            ("Date & time", "Date and time of the celebration?"),
            ("Venue",       "Where is the party? (venue name + address or 'TBD')"),
            ("Style",       "What vibe? (e.g. elegant, party, luxury, modern, neon)"),
            ("Colors",      "Any preferred colors or theme?"),
            ("Contact",     "RSVP contact — phone, email, or WhatsApp?"),
            ("Logo/Image",  "Any photo or logo to include? (paste URL or describe it)"),
        ]
    } else if m.contains("business") || m.contains("product") || m.contains("sale")
           || m.contains("promo") {
        vec![
            ("Business name", "Business name and tagline?"),
            ("Offer",         "What are you promoting? (product, service, or sale offer)"),
            ("Price",         "Any price or discount to show?"),
            ("Contact",       "Phone, email, website, or social handle?"),
            ("Deadline",      "Any expiry date? (e.g. 'Offer ends 30 April')"),
            ("Style",         "Style? (e.g. corporate, bold, modern, luxury)"),
            ("Logo",          "Logo URL or file path? (or describe it)"),
        ]
    } else if m.contains("event") || m.contains("concert") || m.contains("conference") {
        vec![
            ("Event name",  "Name of the event?"),
            ("Date & time", "Date and time?"),
            ("Venue",       "Venue name and address?"),
            ("Ticket",      "Ticket price, or is it free?"),
            ("Contact",     "Contact for tickets or more info?"),
            ("Style",       "Style? (e.g. bold, elegant, energetic)"),
            ("Logo",        "Event logo or organiser logo? (URL or description)"),
        ]
    } else {
        vec![
            ("Title",    "What is the main title of the flyer?"),
            ("Details",  "Key details to include (date, address, price, contact)?"),
            ("Style",    "Style preference? (modern, elegant, bold, playful, luxury)"),
            ("Colors",   "Any specific colors or brand colors?"),
            ("Logo",     "Logo or image to include? (URL or describe)"),
        ]
    }
}

pub fn flyer_pipeline_intro() -> &'static str {
r#"
  🐉 I can build that flyer for you.

  Here's exactly what happens:

  1. You answer a few quick questions (takes 2 minutes)
  2. I search and download relevant background images from the web
  3. I download the fonts that match your chosen style
  4. I load any logo or images you provide
  5. I then open Photoshop and place everything automatically
     — or use my built-in renderer if Photoshop isn't installed
  6. You get: PNG (for sharing) + PDF (for printing) + SVG (for editing)

  All assets are gathered BEFORE Photoshop opens.
  The whole design runs automatically once I have your answers.

  Let's start. You can answer all at once or one by one:
"#
}
