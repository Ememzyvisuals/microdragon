# MICRODRAGON — Complete Abilities Breakdown
## Every Capability, With Real Examples
### by EMEMZYVISUALS DIGITALS — Emmanuel Ariyo

> **MICRODRAGON** = **N**etworked **E**xecution & **C**ognitive **A**utonomy

---

> This document shows every single thing MICRODRAGON can do, with real command examples,
> what the output looks like, and real-world use cases.
> MICRODRAGON handles tasks that used to require 5 different tools, 3 subscriptions,
> and hours of manual work.

---

## Table of Contents

1. [AI Conversation & Reasoning](#1-ai-conversation--reasoning)
2. [Code — Generate, Debug, Review, Test](#2-code--generate-debug-review-test)
3. [Research & Web Intelligence](#3-research--web-intelligence)
4. [Design & Graphics](#4-design--graphics)
5. [Spreadsheets & Data](#5-spreadsheets--data)
6. [Documents & Reports](#6-documents--reports)
7. [Browser Automation](#7-browser-automation)
8. [Desktop Automation](#8-desktop-automation)
9. [WhatsApp Integration](#9-whatsapp-integration)
10. [Telegram Bot](#10-telegram-bot)
11. [Discord Bot](#11-discord-bot)
12. [Voice — Listen & Speak](#12-voice--listen--speak)
13. [Email Management](#13-email-management)
14. [Calendar Management](#14-calendar-management)
15. [GitHub Integration](#15-github-integration)
16. [Business & Finance](#16-business--finance)
17. [File Intelligence](#17-file-intelligence)
18. [Background Watch Daemon](#18-background-watch-daemon)
19. [Skills & Plugins](#19-skills--plugins)
20. [Security & Privacy](#20-security--privacy)
21. [Self-Improvement System](#21-self-improvement-system)
22. [Cost Optimization](#22-cost-optimization)
23. [7 Specialist Agents](#23-7-specialist-agents)
24. [Real-World Scenarios](#24-real-world-scenarios)

---

## 1. AI Conversation & Reasoning

MICRODRAGON uses the world's best AI models to reason, plan, and respond with full context
of everything you've discussed in this session.

### Commands

```bash
# Start interactive session — the most powerful mode
microdragon

# One-shot question
microdragon ask "explain the difference between TCP and UDP simply"

# Stream response token by token (live output)
microdragon ask --stream "write me a complete business plan for a SaaS startup"

# JSON output (for scripting)
microdragon ask --output json "what is 15% of 4200?"

# Force a specific specialist agent
microdragon ask --agent researcher "what happened in AI this week?"
microdragon ask --agent writer "write me a cold email to a potential client"
```

### What it looks like

```
⬡ MICRODRAGON  v0.1.0
────────────────────────────────────────────────────

⬡ #1 ▸ you
explain how a Rust borrow checker works

  ⠋ thinking...
  ▸ executing
  ✓ Done

MICRODRAGON ▸
──────────────────────────────────────────────────
The Rust borrow checker is a compile-time analysis
system that enforces memory safety without garbage
collection. It works on three rules:

1. Each value has exactly ONE owner
2. You can have MANY immutable references (&T)
   OR ONE mutable reference (&mut T) — never both
3. References must not outlive the data they point to

Example:
  let s1 = String::from("hello");
  let s2 = s1;        // s1 MOVED to s2
  println!("{}", s1); // ← COMPILE ERROR: s1 moved

This prevents use-after-free, double-free, and
data races — at zero runtime cost.
──────────────────────────────────────────────────
  ▸ anthropic/claude-opus-4-6 • 412 tokens • 1.2s
```

### Real-world uses

| Task | Command |
|---|---|
| Explain a concept | `microdragon ask "explain machine learning to a 12-year-old"` |
| Write anything | `microdragon ask --stream "write a cover letter for a software engineer role"` |
| Solve problems | `microdragon ask "I have a chicken, fox, and grain river puzzle"` |
| Translate text | `microdragon ask "translate this to French: Hello, how are you?"` |
| Summarise | `microdragon ask "summarise this in 3 bullet points: [paste text]"` |
| Plan anything | `microdragon ask "plan a 7-day trip to Japan for 2 people, $3000 budget"` |

---

## 2. Code — Generate, Debug, Review, Test

The Coder agent writes production-ready code with imports, error handling,
tests, and usage examples. It debugs by reading your actual file and identifying
every issue. It reviews your entire codebase. It integrates with Git.

### Generate code

```bash
# Generate a complete file
microdragon code generate "user authentication with JWT tokens in Express.js"
microdragon code generate "REST API for a todo app" --language python --output api.py
microdragon code generate "binary search implementation" --language rust --output search.rs
microdragon code generate "websocket server with rooms" --language node

# Generate with context
microdragon ask --agent coder "add rate limiting to this Express app: [paste code]"
```

### Output example

```bash
$ microdragon code generate "Redis cache wrapper in Python"

  ⠋ thinking... planning... executing...
  ✓ Code generated (87 lines)

File: microdragon_gen_a3f82b19.py
────────────────────────────────────────
import redis
import json
import functools
from typing import Any, Optional, Callable
from datetime import timedelta

class RedisCache:
    """
    Thread-safe Redis cache wrapper with JSON serialisation,
    TTL support, and decorator pattern.

    Usage:
        cache = RedisCache(host='localhost', port=6379)
        
        @cache.cached(ttl=300)
        def get_user(user_id: str) -> dict:
            return db.query_user(user_id)
    """
    
    def __init__(self, host: str = 'localhost', port: int = 6379,
                 db: int = 0, password: Optional[str] = None):
        self.client = redis.Redis(
            host=host, port=port, db=db,
            password=password, decode_responses=True
        )
    
    def get(self, key: str) -> Optional[Any]:
        try:
            value = self.client.get(key)
            return json.loads(value) if value else None
        except (redis.RedisError, json.JSONDecodeError):
            return None
    
    ... (87 lines total)
```

### Debug code

```bash
microdragon code debug broken_script.py
microdragon code debug api/routes.ts --language typescript
microdragon code debug main.rs
```

```
$ microdragon code debug payment_service.py

  ✓ Analysis complete — 3 issues found

MICRODRAGON Code Review: payment_service.py
────────────────────────────────────────────────────

🚨 CRITICAL — Line 47
  SQL Injection vulnerability
  Code:    query = f"SELECT * FROM users WHERE id = {user_id}"
  Fix:     query = "SELECT * FROM users WHERE id = %s"
           cursor.execute(query, (user_id,))

⚠  HIGH — Line 83
  Unhandled exception: stripe.error.CardError not caught
  If payment fails, the function crashes silently.
  Add: except stripe.error.CardError as e: return {"error": str(e)}

ℹ  LOW — Line 12
  `import *` from config is bad practice.
  Replace: from config import STRIPE_KEY, DATABASE_URL

────────────────────────────────────────────────────
Fixed version written to: payment_service_fixed.py
```

### Review code

```bash
microdragon code review src/main.rs
microdragon code review . --recursive         # entire project
microdragon code review app.py --security     # security-focused
```

```
$ microdragon code review --recursive .

  Scanning 23 files...
  ✓ Review complete

Project Code Review — my_project/
═══════════════════════════════════════════════════

Security: 2 issues
  • [HIGH] Hardcoded API key in config.js:14
  • [MEDIUM] User input not sanitised in search.py:67

Performance: 1 issue
  • [MEDIUM] N+1 database query in users.py:102-115
    (34 queries per page load — use select_related())

Code Quality: 3 issues
  • Functions > 50 lines: auth.py:process_login (87 lines)
  • Missing error handling: api/webhooks.py:receive_payment
  • Unused imports: 4 files

Tests: Missing coverage
  • auth.py: 0% coverage
  • payments.py: 12% coverage (only happy path)

Overall Score: 6.2/10
Recommendation: Fix security issues before deployment.
```

### Git assistance

```bash
microdragon code git "write a commit message"
microdragon code git "explain what changed in the last commit"
microdragon code git "what does this diff mean" --staged
```

```
$ microdragon code git "write a commit message" --staged

  ✓ Analysed staged changes (4 files, +127/-43 lines)

feat(auth): add JWT refresh token rotation

- Add refresh token endpoint with 7-day expiry
- Implement token blacklist using Redis
- Fix session not persisting across page refresh
- Add unit tests for token validation (12 tests)

BREAKING CHANGE: /api/auth/login now returns
  { accessToken, refreshToken } instead of { token }
```

---

## 3. Research & Web Intelligence

The Researcher agent searches the web, crawls multiple sources,
scores relevance, extracts key facts, and synthesises everything
into a structured report — all in one command.

```bash
microdragon research "latest Rust async programming patterns 2025"
microdragon research "compare PostgreSQL vs MongoDB for a social app" --sources 12
microdragon research "OpenAI vs Anthropic product comparison 2026" --output report.md
microdragon research "how to raise a seed round for a startup" --sources 8
```

### Output example

```
$ microdragon research "best practices for API rate limiting 2026"

  ⠋ Searching... crawling 8 sources... extracting... synthesising...
  ✓ Research complete — 8 sources, 3,200 words analysed

───────────────────────────────────────────────────────────
RESEARCH REPORT: API Rate Limiting Best Practices 2026
───────────────────────────────────────────────────────────

EXECUTIVE SUMMARY
Rate limiting in 2026 has evolved beyond simple request counts.
Modern approaches combine token bucket algorithms with AI-driven
anomaly detection and adaptive limits per user tier.

KEY FINDINGS

1. Token Bucket > Fixed Window
   Fixed window allows burst abuse at window boundaries.
   Token bucket (used by Stripe, GitHub) prevents this.

2. Redis is the standard for distributed rate limiting
   lua scripts ensure atomic increment+check operations.
   Average latency added: 0.3ms.

3. Return standard headers every response:
   X-RateLimit-Limit: 1000
   X-RateLimit-Remaining: 847
   X-RateLimit-Reset: 1735689600
   Retry-After: 3600 (when limited)

4. Tier-based limits outperform flat limits
   Free: 100/hour | Pro: 10,000/hour | Enterprise: unlimited

SOURCES
[1] Stripe Engineering Blog — stripe.com/blog
[2] GitHub REST API Documentation — docs.github.com
[3] Cloudflare Rate Limiting Guide — developers.cloudflare.com
[4] Redis Best Practices — redis.io/docs
... (8 sources total)

Saved to: research_api_rate_limiting.md
```

---

## 4. Design & Graphics

MICRODRAGON controls Photoshop via ExtendScript, GIMP via Script-Fu,
or generates programmatically with Pillow. No design software?
It generates HTML/CSS designs that look professional.

```bash
# Image creation (uses best available: Photoshop → GIMP → Pillow)
microdragon design create "dark cyberpunk logo 1920x1080" --output logo.png
microdragon design create "company banner for ememzyvisuals.com" --output banner.png
microdragon design create "YouTube thumbnail with bold text MICRODRAGON IS HERE" --output thumb.png

# Edit existing image
microdragon design edit product.jpg "remove background, make transparent"
microdragon design edit photo.jpg "increase brightness 20%, add vignette"
microdragon design edit logo.png "resize to 512x512, sharpen"

# Color palettes
microdragon design palette "#1a1a2e" --style analogous
microdragon design palette "#ff6b35" --style complementary

# Web design (generates complete HTML/CSS)
microdragon design html "dark SaaS landing page with pricing table"
microdragon design html "mobile-first portfolio for a developer"
microdragon design html "admin dashboard with sidebar navigation"

# SVG graphics
microdragon design svg "abstract geometric pattern for website background"
microdragon design svg "simple icon: shield with checkmark"
```

### Palette output example

```
$ microdragon design palette "#2563eb" --style analogous

  ✓ Palette generated

Analogous Palette — Base: #2563eb
═══════════════════════════════

  ████  #1e40af  — Deep Blue (shadow/dark)
  ████  #1d4ed8  — Royal Blue (primary dark)
  ████  #2563eb  — Cornflower Blue (primary) ← base
  ████  #3b82f6  — Sky Blue (primary light)
  ████  #93c5fd  — Pale Blue (highlight/text)

CSS Variables:
:root {
  --color-primary:    #2563eb;
  --color-primary-dk: #1d4ed8;
  --color-primary-lt: #3b82f6;
  --color-shadow:     #1e40af;
  --color-highlight:  #93c5fd;
}

Tailwind equivalent: blue-600, blue-700, blue-500, blue-800, blue-300
```

---

## 5. Spreadsheets & Data

MICRODRAGON creates Excel spreadsheets with formatting, charts, auto-filters,
and freeze panes. Works with Excel, LibreOffice Calc, or openpyxl (no app needed).

```bash
# Create spreadsheets
microdragon spreadsheet create "monthly budget tracker 2026" --output budget.xlsx
microdragon spreadsheet create "sales performance Q1" --charts --output sales.xlsx
microdragon spreadsheet create "employee directory with departments" --output staff.xlsx

# Import data and create spreadsheet
microdragon spreadsheet create "crypto portfolio tracker" --data portfolio.json --output crypto.xlsx

# Analyse existing data
microdragon spreadsheet analyse sales_data.csv "find monthly trends and top performers"
microdragon spreadsheet analyse revenue.xlsx "calculate year-over-year growth"

# Modify existing spreadsheet
microdragon spreadsheet open budget.xlsx "add a monthly totals row and highlight over-budget cells"
```

### What MICRODRAGON creates

```
$ microdragon spreadsheet create "sales tracker Q1 2026" --charts

  ⠋ Creating spreadsheet...
  ✓ Created: sales_tracker_q1.xlsx

Contents:
  Sheet 1: Data (187 formatted rows)
  ├── Headers: bold, dark blue background, white text
  ├── Alternating row colours for readability
  ├── Auto-filter on all columns
  ├── Frozen header row
  └── Auto-fit column widths

  Sheet 2: Charts
  ├── Bar chart: Monthly Revenue by Region
  └── Line chart: Cumulative vs Target

  Sheet 3: Summary
  ├── Total rows: 187
  ├── Generated: 2026-04-07
  └── Source: MICRODRAGON AI

Open: libreoffice --calc sales_tracker_q1.xlsx
```

---

## 6. Documents & Reports

MICRODRAGON creates formatted Word documents with headings, tables, footers,
and professional styling. Also reads any document and answers questions.

```bash
# Create documents
microdragon document create "Q3 Financial Report for EMEMZYVISUALS DIGITALS" --output q3_report.docx
microdragon document create "Non-Disclosure Agreement template" --style legal --output nda.docx
microdragon document create "Job description: Senior Rust Engineer" --output jd_rust.docx
microdragon document create "Invoice template" --output invoice.docx

# Read and query documents
microdragon document read contract.pdf "what are the payment terms and penalties?"
microdragon document read annual_report.pdf "what was the net profit in 2025?"
microdragon document read thesis.pdf "summarise the key findings in 200 words"
microdragon document read employees.xlsx "who earns the most?"
microdragon document read meeting_notes.docx "what action items were agreed?"

# Summarise entire folders
microdragon document summarise ~/Downloads/

# Convert between formats
microdragon document convert notes.md --output notes.docx
microdragon document convert presentation.pptx  # extract text
```

### Document creation output

```
$ microdragon document create "Technical Architecture Document for MICRODRAGON" --output arch.docx

  ✓ Document created: arch.docx (4,200 words)

Document Structure:
  Page 1:    Title page with MICRODRAGON branding
  Section 1: Executive Summary
  Section 2: System Architecture Overview
             └── Architecture diagram (text-based)
  Section 3: Core Components
             ├── Rust Engine
             ├── Brain Layer
             ├── Module System
             └── Security Layer
  Section 4: Data Flow
  Section 5: Security Model
  Section 6: Deployment Guide
  Footer:    "MICRODRAGON AI Agent | 2026-04-07 | Confidential"

Open: libreoffice --writer arch.docx
```

---

## 7. Browser Automation

MICRODRAGON uses Playwright to control a real Chrome browser.
It can log in, fill forms, extract data, click buttons, download files,
and automate any web workflow.

```bash
# Data extraction
microdragon automate browser --url "https://news.ycombinator.com" "extract all story titles and scores"
microdragon automate browser "go to linkedin.com/jobs and find remote Python jobs posted today"
microdragon automate browser "get the current price of AAPL from finance.yahoo.com"

# Form automation
microdragon automate browser "go to typeform.com/signup and fill the registration form with test data"
microdragon automate browser "log into github.com and star the repository ememzyvisuals/microdragon"

# Screenshots
microdragon automate browser "screenshot the homepage of ememzyvisuals.com" --headless

# Downloads
microdragon automate browser --url "https://example.com/reports" "download all PDF files on this page"

# Monitoring
microdragon automate schedule "check competitor pricing on shopify.com/pricing every Monday morning" --cron "0 9 * * 1"
```

### Output example

```
$ microdragon automate browser --url "https://hn.algolia.com" "get top 10 AI stories today"

  ⠋ Launching browser... navigating... extracting...
  ✓ Extracted 10 stories

Top 10 AI Stories on Hacker News Today
════════════════════════════════════════
1. [847 pts] "Anthropic releases Claude 4 Opus with 1M context"
   URL: news.ycombinator.com/item?id=42847291

2. [634 pts] "Why I switched from Python to Rust for ML inference"
   URL: news.ycombinator.com/item?id=42841823

3. [521 pts] "MICRODRAGON: The first AI agent that can operate any desktop app"
   URL: news.ycombinator.com/item?id=42839901

... (10 stories total)

Completed in: 4.2s
```

---

## 8. Desktop Automation

MICRODRAGON uses PyAutoGUI and OpenCV to control your actual desktop.
It can click buttons, type text, take screenshots, find elements on screen,
and operate any GUI application — including Photoshop, Excel, Slack, everything.

```bash
# Screenshot operations
microdragon automate desktop "take a screenshot of the entire screen"
microdragon automate desktop "screenshot just the top-right corner (0,0 to 500,400)"

# Keyboard and mouse
microdragon automate desktop "press Ctrl+S to save the current document"
microdragon automate desktop "type this text in the active window: Dear Customer,"
microdragon automate desktop "move the mouse to position 960 540 and click"
microdragon automate desktop "press Alt+Tab to switch windows"

# App control
microdragon automate desktop "open Calculator app and compute 1337 multiplied by 42"
microdragon automate desktop "open Photoshop and create a new 1920x1080 canvas"
microdragon automate desktop "minimize all windows"

# Multi-step workflows
microdragon automate workflow daily_standup.json
```

### Workflow file example

```json
// daily_standup.json
{
  "name": "Daily Standup Prep",
  "steps": [
    { "action": "screenshot", "params": { "path": "/tmp/screen.png" } },
    { "action": "hotkey", "params": { "keys": ["ctrl", "alt", "t"] } },
    { "action": "wait", "params": { "seconds": 1 } },
    { "action": "type", "params": { "text": "git log --oneline -10\n" } },
    { "action": "wait", "params": { "seconds": 2 } },
    { "action": "screenshot", "params": { "path": "/tmp/git_log.png" } }
  ]
}
```

```
$ microdragon automate workflow daily_standup.json

  Running: Daily Standup Prep (6 steps)
  ✓ Step 1: Screenshot taken → /tmp/screen.png
  ✓ Step 2: Terminal opened (Ctrl+Alt+T)
  ✓ Step 3: Waited 1s
  ✓ Step 4: Typed git log command
  ✓ Step 5: Waited 2s
  ✓ Step 6: Git log screenshot → /tmp/git_log.png
  
  Workflow complete: 6/6 steps succeeded
```

---

## 9. WhatsApp Integration

MICRODRAGON connects to your WhatsApp account via QR code (no API key, no approval).
Once connected, it listens to all messages. Any message becomes a MICRODRAGON command.
It remembers context per-user and responds like a real AI assistant.

### Setup

```bash
microdragon social whatsapp start
```

```
  ▸ Starting WhatsApp bridge...

  Scan this QR code with WhatsApp:

  ██████████████████████████████████
  █ ▄▄▄▄▄ █▀ █▀▀██▀▄▀ █▀▄█ ▄▄▄▄▄ █
  █ █   █ █▀▀▄█▄▀█▄▀▀▄███ █ █   █ █
  █ █▄▄▄█ █▀▄▀▄▄█▀▄▄▄▀▄▀▄ █ █▄▄▄█ █
  ▀ ▀▀▀▀▀ ▀▄▀ █▄█▀█▄▀▄▄▀▄▀▀▀▀▀▀▀▀ ▀
  ... (QR code)
  ██████████████████████████████████

  Open WhatsApp → Settings → Linked Devices → Link a Device
  ✓ Connected as: +44 7700 900000

  Listening for messages...
```

### Conversation example

```
[WhatsApp — You → MICRODRAGON Bot]

You:    write me a Python function to merge two sorted lists

MICRODRAGON:   Here's a clean Python implementation:

        def merge_sorted(list1: list, list2: list) -> list:
            """Merge two sorted lists into one sorted list.
            Time: O(n+m), Space: O(n+m)"""
            result = []
            i, j = 0, 0
            while i < len(list1) and j < len(list2):
                if list1[i] <= list2[j]:
                    result.append(list1[i]); i += 1
                else:
                    result.append(list2[j]); j += 1
            result.extend(list1[i:])
            result.extend(list2[j:])
            return result

        # Test:
        merge_sorted([1,3,5], [2,4,6])
        → [1, 2, 3, 4, 5, 6]

You:    research the latest AI agent frameworks

MICRODRAGON:   Research complete (6 sources):

        Top AI Agent Frameworks 2026:
        1. MICRODRAGON — Rust core, 7 agents, full OS control
        2. Claude Code — coding-focused, $2.5B ARR
        3. OpenClaw — messaging-first, Node.js
        4. LangGraph — graph-based workflow
        5. AutoGen — Microsoft multi-agent
        ...
```

---

## 10. Telegram Bot

Full CLI parity in Telegram. Every MICRODRAGON command available as a Telegram message.
Supports slash commands, streaming responses, per-user context memory.

### Setup

```bash
# 1. Create bot via @BotFather in Telegram
# 2. Configure
microdragon config set-key telegram 1234567890:TOKEN
microdragon social telegram start
```

### Conversation example

```
[Telegram — @my_microdragon_bot]

/start

🤖 MICRODRAGON Universal AI Agent
━━━━━━━━━━━━━━━━━━━━━━━━
Your personal AI team, always online.

Commands:
/help      — show commands
/code      — code generation
/research  — web research
/status    — system status
/clear     — clear history

Or just type anything!

──────────────────────

/code build a REST API with FastAPI and SQLite

⚙️ Coding agent activated...

Here's a complete FastAPI + SQLite REST API:

```python
from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
import sqlite3
from contextlib import contextmanager

app = FastAPI(title="MICRODRAGON API")
DB_PATH = "data.db"

@contextmanager
def get_db():
    conn = sqlite3.connect(DB_PATH)
    conn.row_factory = sqlite3.Row
    try:
        yield conn
        conn.commit()
    finally:
        conn.close()
...
```

[17 files were sent to Telegram in chunks]

──────────────────────

research quantum computing breakthroughs 2026

🔍 Researching from 8 sources...

**Quantum Computing — Key 2026 Breakthroughs:**

• IBM achieved 1000+ qubit stability for 10ms
• Google announced room-temperature quantum processor
• Chinese team broke 2048-bit RSA encryption in lab
• D-Wave ships commercial 10,000-qubit machine

[Full 800-word report with sources follows...]
```

---

## 11. Discord Bot

Same power in Discord. Uses prefix commands in any channel MICRODRAGON has access to.

```bash
microdragon config set-key discord YOUR_TOKEN
microdragon social discord start
```

### Discord usage

```
[#dev-channel on Your Server]

You:  !microdragon code write a Discord.js slash command for /ping

MICRODRAGON: Here's a complete Discord.js slash command:

      const { SlashCommandBuilder } = require('discord.js');
      
      module.exports = {
        data: new SlashCommandBuilder()
          .setName('ping')
          .setDescription('Check bot latency'),
        
        async execute(interaction) {
          const sent = await interaction.reply({
            content: 'Pinging...', fetchReply: true
          });
          await interaction.editReply(
            `Pong! Latency: ${sent.createdTimestamp - 
            interaction.createdTimestamp}ms`
          );
        },
      };

You:  !microdragon research best Discord bot hosting options 2026

MICRODRAGON: **Discord Bot Hosting 2026:**

      🥇 Railway — $5/month, auto-deploy from GitHub
      🥈 Fly.io — free tier available, global edge
      🥉 DigitalOcean Droplet — $6/month, full control
      ⭐ Oracle Cloud Free — completely free forever (2 VMs)

      [Full comparison with uptime, pricing, features...]

You:  !microdragon clear

MICRODRAGON: ✅ Conversation context cleared.
```

---

## 12. Voice — Listen & Speak

MICRODRAGON listens to your microphone, transcribes with Whisper,
processes with AI, and speaks back using ElevenLabs, OpenAI TTS,
or your system's built-in TTS (macOS/Windows/Linux — all supported free).

```bash
# One-time voice interaction
microdragon voice listen                    # record 5s then respond
microdragon voice listen --duration 15      # record 15s

# Text to speech
microdragon voice say "Good morning. You have 3 emails and 2 meetings today."
microdragon voice say "Task complete." --voice nova    # OpenAI voice
microdragon voice say "Done." --voice rachel           # ElevenLabs voice

# Transcribe audio file
microdragon voice transcribe meeting_recording.mp3
microdragon voice transcribe lecture.wav --output transcript.txt

# Wake word mode (always listening)
microdragon voice wake
# Say "hey microdragon" → MICRODRAGON activates → listen → respond → back to standby
```

### Wake word session

```
$ microdragon voice wake

  🎤 Wake word detection active
  Say "hey microdragon" to activate

  [You say: "hey microdragon, what's the weather in London?"]

  Wake word detected ✓
  Listening... (recording 5s)
  Transcribed: "what's the weather in London?"
  Processing...

  [MICRODRAGON speaks]:
  "London is currently 14 degrees Celsius, partly cloudy,
   with rain expected this afternoon. Bring an umbrella."

  Returning to standby...
  Say "hey microdragon" to activate again
```

### TTS options (by quality, all work)

| Engine | Cost | Quality | Setup |
|---|---|---|---|
| ElevenLabs | $0.30/1k chars | ⭐⭐⭐⭐⭐ | Set `ELEVENLABS_API_KEY` |
| OpenAI TTS | $0.015/1k chars | ⭐⭐⭐⭐ | Uses `OPENAI_API_KEY` |
| Piper (local) | Free | ⭐⭐⭐ | `pip install piper-tts` |
| macOS `say` | Free | ⭐⭐⭐ | Built-in, no setup |
| Windows SAPI | Free | ⭐⭐ | Built-in, no setup |
| Linux espeak | Free | ⭐⭐ | `apt install espeak` |

---

## 13. Email Management

MICRODRAGON reads your inbox and drafts replies. The critical difference from OpenClaw:
**MICRODRAGON never sends email without showing you a full preview and getting an explicit yes.**

```bash
# Setup (Gmail example)
export MICRODRAGON_EMAIL="you@gmail.com"
export MICRODRAGON_EMAIL_PASSWORD="your_app_password"
# App password: Gmail Settings → Security → 2FA → App Passwords

# Read inbox
microdragon email inbox                    # summarise unread emails
microdragon email inbox --limit 50         # last 50 unread
microdragon email search "invoice"         # search by subject
microdragon email search "from: boss@co.com"

# Draft and send (ALWAYS shows preview first)
microdragon email compose \
  --to client@company.com \
  --subject "Project Update — Week 15" \
  --body "Hi Sarah, here's this week's progress report..."
```

### Email compose flow

```
$ microdragon email compose --to ceo@company.com --subject "Proposal" --body "..."

┌──────────────────────────────────────────────────────┐
│  EMAIL DRAFT — REQUIRES YOUR CONFIRMATION             │
├──────────────────────────────────────────────────────┤
│  To:      ceo@bigcompany.com                         │
│  CC:      none                                       │
│  Subject: MICRODRAGON Integration Proposal                  │
├──────────────────────────────────────────────────────┤
  Dear Mr Johnson,

  I wanted to reach out regarding a proposal to integrate
  MICRODRAGON AI Agent into your development workflow...

  [Full email body shown here]

  Best regards,
  Emmanuel Ariyo
  EMEMZYVISUALS DIGITALS
├──────────────────────────────────────────────────────┤
│  Attachments: 0 files                                │
└──────────────────────────────────────────────────────┘

⚠  Type 'yes' to send, anything else to cancel: yes

  ✓ Email sent to ceo@bigcompany.com
```

---

## 14. Calendar Management

MICRODRAGON reads and creates calendar events. Works with Google Calendar (API)
and macOS Calendar (AppleScript). Generates AI morning briefings.

```bash
# View events
microdragon calendar today
microdragon calendar week
microdragon calendar month

# AI briefing (great for daily automation)
microdragon calendar briefing

# Create events (always shows confirmation)
microdragon calendar add "Team standup" "2026-04-15 09:00" --duration 30
microdragon calendar add "Client call with Johnson & Co" "tomorrow 14:00" \
                  --location "Zoom: https://zoom.us/j/123" \
                  --notes "Bring the Q3 proposal"
```

### Morning briefing example

```
$ microdragon calendar briefing

  ✓ Calendar loaded (Google Calendar)

📅 Tuesday, April 7, 2026 — Morning Briefing

TODAY'S SCHEDULE (4 events)
  09:00  Team Standup                    (30 min) — Google Meet
  11:00  1:1 with Sarah                  (45 min) — Office
  14:00  Client Call: Johnson & Co       (60 min) — Zoom
  17:30  Product Demo Prep               (90 min) — Office

IMPORTANT NOTES
  • Client call at 14:00: bring Q3 proposal and pricing sheet
  • 3 meetings back-to-back from 09:00-12:45, no lunch gap

UPCOMING THIS WEEK
  Thu: All-hands company meeting 10:00
  Fri: Sprint review + retrospective 14:00

AI SUMMARY
  Heavy meeting day. Focus deep work before 09:00 or after 17:30.
  Block 30 minutes after the client call to capture next steps.
```

---

## 15. GitHub Integration

MICRODRAGON reviews pull requests with actual code analysis — security vulnerabilities,
logic errors, performance issues. Creates issues. Gives repo overviews.

```bash
# Set up
export GITHUB_TOKEN="ghp_your_token_here"

# Review a pull request
microdragon github review https://github.com/owner/repo/pull/123

# Post the review directly to GitHub (adds as PR comment)
microdragon github review https://github.com/owner/repo/pull/123 --post

# Create an issue
microdragon github issue ememzyvisuals/microdragon "Bug: voice module crashes on macOS 13"
microdragon github issue ememzyvisuals/microdragon "Feature request: add Signal integration"

# Get repo overview
microdragon github overview ememzyvisuals/microdragon

# List open PRs
microdragon github prs torvalds/linux
```

### PR Review output

```
$ microdragon github review https://github.com/mycompany/api/pull/47

  ✓ PR loaded: 12 files, +284/-67 lines
  ✓ Static analysis complete
  ✓ AI review complete

══════════════════════════════════════════════════════
PR #47 — feat: add user subscription management
Author: dev_junior | main ← feature/subscriptions
══════════════════════════════════════════════════════

VERDICT: REQUEST CHANGES

SUMMARY
Overall solid implementation, but there's a critical SQL injection
vulnerability and a missing uniqueness constraint that will cause
duplicate subscription issues in production.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

🚨 CRITICAL — src/subscriptions/db.py:47
   SQL Injection vulnerability
   
   Code:
     query = f"SELECT * FROM subs WHERE user = '{user_id}'"
   
   Exploit:
     user_id = "'; DROP TABLE subs; --"
   
   Fix:
     query = "SELECT * FROM subs WHERE user = %s"
     cursor.execute(query, (user_id,))

⚠  HIGH — migrations/003_subscriptions.sql:12
   Missing UNIQUE constraint
   
   users can end up with multiple active subscriptions
   for the same plan due to race condition in signup flow.
   
   Add: UNIQUE(user_id, plan_id, status) WHERE status='active'

ℹ  LOW — src/subscriptions/views.py:23-67
   Function is 45 lines — consider extracting validate_subscription()
   and process_payment() as separate functions for testability.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Tests: Only happy path covered. Add tests for:
  • Duplicate subscription attempt
  • Expired payment method
  • Plan downgrade during active period

Review posted to GitHub ✓
```

---

## 16. Business & Finance

The Analyst agent fetches live market data from Yahoo Finance,
computes RSI, MACD, moving averages, support/resistance levels,
and gives a BUY/SELL/HOLD signal with confidence score.

```bash
microdragon business market AAPL
microdragon business market BTC-USD --interval 1h
microdragon business market NVDA --detailed         # all technical indicators
microdragon business signal SPY                     # BUY/SELL/HOLD
microdragon business signal ETH-USD --confidence    # include confidence score
microdragon business risk TSLA                      # risk assessment
microdragon business portfolio                      # portfolio analysis
microdragon business report AAPL --output report.pdf
```

### Market analysis output

```
$ microdragon business market AAPL

  ✓ Market data loaded — 90 days history

MARKET ANALYSIS: AAPL (Apple Inc.)
════════════════════════════════════════════════

PRICE           $184.32  ▲ +$2.14 (+1.17%)
52-Week High    $199.62
52-Week Low     $164.08
Market Cap      $2.84T
Volume          58,234,100

TECHNICAL INDICATORS
  RSI (14):          61.4    (Neutral — approaching overbought)
  MACD:              Bullish crossover (2 days ago)
  Trend:             Uptrend (confirmed by 3 consecutive higher lows)
  SMA 20:            $181.44
  SMA 50:            $177.21
  EMA 12:            $183.91

SUPPORT & RESISTANCE
  Strong Resistance: $188.00 (previous high)
  Immediate Support: $181.50 (SMA 20)
  Key Support:       $177.00 (SMA 50)

SIGNAL: WATCH → BUY ON PULLBACK
  Confidence: 71%
  
  Rationale:
  • Bullish MACD crossover is a positive signal
  • RSI approaching 70 — wait for slight pullback before entering
  • Strong support at $181 — consider buying there if it pulls back
  • Uptrend intact: higher highs and higher lows over 3 weeks

RISK NOTE: Past performance does not predict future results.
This is analysis only, not financial advice.
```

---

## 17. File Intelligence

MICRODRAGON reads any file type and answers questions about it using AI.
Supports PDF, Word, Excel, CSV, PowerPoint, plain text, code files.

```bash
# Read any document and ask questions
microdragon files read contract.pdf "what are the termination clauses?"
microdragon files read annual_report.pdf "what was net profit in 2025 vs 2024?"
microdragon files read data.xlsx "which region has the highest sales?"
microdragon files read codebase.py "explain what this code does in simple terms"
microdragon files read meeting_notes.txt "what were the key decisions made?"

# Summarise
microdragon files read whitepaper.pdf --summarise
microdragon files read large_log.txt --summarise

# Summarise an entire folder
microdragon files summarise ~/Downloads/          # reads all docs, gives overview
microdragon files summarise ~/Projects/           # reads code files, gives summary

# Semantic search across files (find by meaning, not just keywords)
microdragon files search "revenue projections" ~/Documents/
microdragon files search "authentication bug" ~/Projects/

# Convert
microdragon files convert notes.md --output notes.docx
microdragon files convert presentation.pptx        # extract all text
```

### File reading output

```
$ microdragon files read services_contract.pdf "what are the payment terms?"

  ✓ PDF loaded: 24 pages, 8,400 words
  ✓ Relevant sections identified
  ✓ Analysis complete

ANSWER: Payment Terms in services_contract.pdf
══════════════════════════════════════════════

From Section 7.2 (Payment Terms):

• Payment due: Net 30 from invoice date
• Late payment penalty: 1.5% per month on overdue amounts
• Currency: GBP (British Pounds)
• Method: Bank transfer only (BACS or SWIFT)
• Disputed invoices: Must be raised within 14 days of receipt

From Section 7.4 (Milestones):
• 30% upfront on contract signing
• 40% on delivery of Phase 1 (due Week 8)
• 30% on final delivery and sign-off

IMPORTANT CLAUSE (Section 7.6):
"Failure to pay within 45 days constitutes material breach
and entitles the Supplier to suspend all services immediately."

Page references: pp. 14-16, 22
```

---

## 18. Background Watch Daemon

MICRODRAGON runs silently in the background, monitoring conditions you define.
When a condition is met, it takes the action you specified.

```bash
# Financial alerts
microdragon watch "alert me when AAPL drops below $150"
microdragon watch "notify on Telegram when BTC crosses $100,000"
microdragon watch "send WhatsApp message when NVDA gains more than 5% in a day"

# Email monitoring
microdragon watch "alert me when a new email arrives from investor@vcfund.com"
microdragon watch "notify when subject contains 'urgent' in my inbox"

# Time-based
microdragon watch "send daily market briefing to Telegram at 8am every weekday"
microdragon watch "remind me every Friday at 4pm to send the weekly report"

# System monitoring
microdragon watch "alert if disk usage on / exceeds 90%"
microdragon watch "notify when RAM usage is over 85% for more than 5 minutes"

# Web monitoring
microdragon watch "check ememzyvisuals.com/pricing hourly and alert if prices change"
microdragon watch "monitor github.com/ememzyvisuals/microdragon daily and report new issues"

# Manage watches
microdragon watch list                    # show all active conditions
microdragon watch remove 3                # remove watch #3
microdragon watch pause 1                 # temporarily disable watch #1
microdragon watch resume 1
```

### Watch list output

```
$ microdragon watch list

Active Watch Conditions (4)
════════════════════════════════════════════════

#1  📈  FINANCIAL   — AAPL drops below $150
    Action:  Send Telegram alert
    Status:  ✓ Active | Checks: every 60s
    Last:    2026-04-07 20:30 | Never triggered

#2  ⏰  SCHEDULE    — Daily market briefing 8am weekdays
    Action:  Send to Telegram + run market analysis
    Status:  ✓ Active | Next: 2026-04-08 08:00
    Last:    2026-04-07 08:00 | Triggered 23 times

#3  📧  EMAIL       — Email from investor@vcfund.com
    Action:  WhatsApp alert + read and summarise email
    Status:  ✓ Active | Checks: every 5 min
    Last:    2026-04-07 20:25 | Triggered 2 times

#4  💻  SYSTEM      — Disk usage > 90%
    Action:  Desktop notification + log to file
    Status:  ✓ Active | Checks: every 60s
    Last:    2026-04-07 20:30 | Never triggered
```

---

## 19. Skills & Plugins

Install custom skills from the community or write your own.
Unlike OpenClaw (Cisco found RCE in unvetted skills), MICRODRAGON scans every skill
for malicious code before installation.

```bash
# List installed skills
microdragon skills list

# Install a skill (security scan runs automatically)
microdragon skills install https://github.com/someone/microdragon-weather-skill/SKILL.md
microdragon skills install https://microdragon.ememzyvisuals.com/skills/news-briefing
microdragon skills install ./my_custom_skill/     # from local directory

# Run a skill
microdragon skills run weather "London"
microdragon skills run news-briefing "technology"
microdragon skills run custom-crm "get this week's leads"

# Manage
microdragon skills remove weather
microdragon skills scan ./my_skill/               # scan without installing
```

### Skill format (write your own)

```markdown
---
name: my-data-puller
version: 1.0.0
description: Pull data from my company's internal API
author: Emmanuel Ariyo
license: MIT
capabilities: network_read, data_extraction
permissions: network
---

# My Data Puller Skill

This skill connects to our internal API and pulls client data.

## Usage
Tell MICRODRAGON: "get client data for account 12345"
```

---

## 20. Security & Privacy

```bash
# View security status
microdragon status

# View audit log (every action MICRODRAGON has taken)
cat ~/.local/share/microdragon/audit.log | tail -20

# Security check
microdragon security audit          # show recent security events
```

### Audit log example

```
$ tail -10 ~/.local/share/microdragon/audit.log

{"action":"CHAT_RESPONSE","detail":"ask: write a Python function","severity":"Info","timestamp":"2026-04-07T20:30:01Z"}
{"action":"FILE_READ","detail":"contract.pdf","severity":"Info","timestamp":"2026-04-07T20:28:45Z"}
{"action":"SHELL_BLOCKED","detail":"rm -rf attempted","severity":"High","timestamp":"2026-04-07T20:15:22Z"}
{"action":"PROMPT_INJECTION_BLOCKED","detail":"Role hijacking: 'ignore previous instructions'","severity":"High","timestamp":"2026-04-07T19:55:11Z"}
{"action":"EMAIL_SENT","detail":"to:client@co.com subject:proposal","severity":"Info","timestamp":"2026-04-07T18:30:00Z"}
{"action":"WEB_RESEARCH","detail":"query: API rate limiting best practices","severity":"Info","timestamp":"2026-04-07T17:44:23Z"}
```

---

## 21. Self-Improvement System

```bash
# Tell MICRODRAGON how it did
microdragon tasks feedback abc123 good      # this response was great
microdragon tasks feedback def456 bad       # this response was wrong

# View your performance stats
microdragon /performance

# In interactive mode
/performance
```

### Performance dashboard

```
/performance

MICRODRAGON Performance Dashboard
═══════════════════════════════════════════════════

SESSION STATS
  Tasks completed:  47
  Success rate:     91.5%  ████████████████████░ 
  Avg latency:      1.4s
  Tokens used:      84,200
  Session cost:     $0.21 (Groq provider)

AGENT PERFORMANCE
  Coder:      38/40 ✓ (95%)   ████████████████████
  Researcher: 12/13 ✓ (92%)  ████████████████████
  Analyst:     8/8  ✓ (100%) ████████████████████
  Automator:   4/6  ✓ (67%)  ██████████████
  Writer:     11/12 ✓ (92%)  ████████████████████

USER FEEDBACK
  👍 Good: 34 | 👎 Bad: 3 | Satisfaction: 91.9%

SELF-IMPROVEMENT LOG (last 5)
  → Coder improved prompt for Rust lifetimes (2 good feedbacks)
  → Researcher now uses 8 sources for complex queries (was 5)
  → Automator adds more error handling after 2 failed scripts
  → Response style matches your preference for code-first answers
```

---

## 22. Cost Optimization

```bash
# Switch to free Groq tier
microdragon config provider groq
microdragon config set-key groq gsk_...     # console.groq.com — free

# Use local Ollama (completely free forever)
ollama pull llama3.1
microdragon config provider custom          # http://localhost:11434/v1

# Enable smart routing
export MICRODRAGON_PREFER_CHEAP=true        # routes simple tasks to cheapest model
export MICRODRAGON_DAILY_BUDGET_USD=0.50    # cap daily spend

# Check session cost
microdragon /cost
```

### Cost breakdown

```
/cost

MICRODRAGON Cost Summary
═══════════════════════════════════════

Session tokens:  84,200
Provider:        Groq (llama-3.3-70b-versatile)
Session cost:    $0.021   ← about 2 cents

Cache hits:      8 (saved ~12,000 tokens = $0.003)
Smart routing:   12 tasks → Groq instant (saved vs Claude: $1.84)

PROVIDER COMPARISON (for today's usage)
  Claude Opus:    $1.26
  Claude Sonnet:  $0.25
  GPT-4o:         $0.21
  Groq 70B:       $0.02   ← what you paid
  Ollama (local): $0.00   ← free option

Set MICRODRAGON_PREFER_LOCAL=true to use Ollama for all simple tasks.
```

---

## 23. Seven Specialist Agents

MICRODRAGON automatically routes each task to the right agent.
You can also force a specific agent.

```bash
microdragon ask --agent coder "..."
microdragon ask --agent researcher "..."
microdragon ask --agent analyst "..."
microdragon ask --agent automator "..."
microdragon ask --agent writer "..."
microdragon ask --agent security "..."
# or let MICRODRAGON choose (default — Master orchestrates)
```

| Agent | Temperature | Best At |
|---|---|---|
| **Master** | 0.7 | Complex multi-domain tasks, coordination |
| **Coder** | 0.2 | Precise code generation, debugging |
| **Researcher** | 0.4 | Web research, source synthesis |
| **Analyst** | 0.3 | Market data, financial analysis |
| **Automator** | 0.1 | Deterministic browser/desktop scripts |
| **Writer** | 0.8 | Creative writing, emails, content |
| **Security** | 0.1 | Code review, vulnerability analysis |

---

## 24. Real-World Scenarios

### Scenario 1: Solo Developer Morning Routine

```bash
# 7:30am — Voice wake word
"hey microdragon, good morning briefing"

MICRODRAGON speaks:
  "Good morning Emmanuel. Today you have 3 meetings starting at 9am.
   AAPL is up 1.2% at $186. You have 5 unread emails, 
   2 from clients. Your GitHub has 1 new PR to review.
   Top HN story: 'MICRODRAGON hits 100k npm downloads'. Good morning."

# 8:00am — Code review
microdragon github review https://github.com/ememzyvisuals/microdragon/pull/142

# 8:30am — Debug a feature
microdragon code debug src/api/subscriptions.py

# 10:00am — Research for a meeting
microdragon research "SaaS pricing strategies for AI tools 2026" --output pricing_research.md

# 3:00pm — Write a proposal
microdragon document create "Partnership Proposal for TechCorp" --output proposal.docx

# 5:00pm — End of day
microdragon tasks history --limit 20    # review what was done
microdragon /performance                # check costs and success rate
```

### Scenario 2: Content Creator

```bash
# Research trending topics
microdragon research "trending AI content ideas YouTube April 2026" --sources 10

# Create thumbnail
microdragon design create "YouTube thumbnail: bold text 'MICRODRAGON AI IS INSANE' fire background"

# Write script
microdragon ask --agent writer --stream "write a 10-minute YouTube script about MICRODRAGON AI agent"

# Schedule posts via WhatsApp/Telegram automation
microdragon watch "remind me every Sunday at 6pm to schedule next week's content"
```

### Scenario 3: E-commerce Business

```bash
# Monitor competitor pricing
microdragon watch "check competitor.com/pricing every 6 hours and alert if they change prices"

# Analyse sales data
microdragon files read sales_q1.xlsx "show me top 10 products by revenue and profit margin"

# Generate reports
microdragon spreadsheet create "Monthly Sales Dashboard" --data sales_data.json --charts

# Customer email drafts
microdragon email compose --to customer@email.com \
  --subject "Your order #12345 has shipped" \
  --body "Hi Sarah, your order is on its way..."
# Shows preview → confirm → sends

# Market research
microdragon research "e-commerce trends UK 2026" --sources 12 --output trends.md
```

### Scenario 4: Student / Researcher

```bash
# Read papers
microdragon files read research_paper.pdf "summarise the methodology and key findings"
microdragon files read 10_papers/*.pdf --summarise   # summarise multiple papers

# Research
microdragon research "latest findings on large language model hallucination 2025-2026" --sources 15

# Write reports
microdragon document create "Literature Review: AI Safety Approaches" --style academic

# Code for data analysis
microdragon code generate "Python script to visualise CSV data with matplotlib and pandas"

# Take notes hands-free
microdragon voice listen --duration 30  # dictate notes, MICRODRAGON transcribes and formats
```

### Scenario 5: Small Business Owner (non-technical)

```bash
# Morning briefing from WhatsApp
[WhatsApp → MICRODRAGON]
"morning briefing"

MICRODRAGON: "Good morning! Quick summary:
  📧 7 new emails — 2 from clients, 1 urgent
  📅 Client call at 2pm today
  📈 Your tracked stocks: AAPL +1.2% NVDA -0.4%
  💡 Task reminder: send invoice to Johnson & Co"

# Quick tasks from phone while commuting
"draft an email to johnson@jco.com saying the invoice is attached and payment is due Friday"

MICRODRAGON shows draft + "reply YES to send"

"research best CRM software for a 10-person sales team under £50/month"

MICRODRAGON: Full comparison of Pipedrive, HubSpot, Zoho, Monday CRM with pricing and features

"create a spreadsheet tracking my 5 clients, invoices, and payment status"

MICRODRAGON: Creates clients_invoices.xlsx with formatted table, colour-coded by status
```

---

## MICRODRAGON vs The Competition — Final Word

```
CLAUDE CODE          OPENCLAW            MICRODRAGON
─────────────────────────────────────────────────────
Code only            Message routing     Everything
Terminal only        Messaging apps      CLI + All platforms
Anthropic only       Multi-model         Any model + Free Ollama
$20+/month           Free (API costs)    Free (API costs / $0 Ollama)
Closed source        MIT                 MIT open source
No desktop control   No desktop control  Opens Photoshop. Controls Excel.
No voice             macOS/iOS voice     All platforms + wake word
No email             Email (risky)       Email with mandatory confirmation
No calendar          Basic calendar      Full calendar + AI briefing
No PR review         No PR review        Full AI code review → posts to GitHub
No document reading  No document AI      Reads any PDF/Word and answers questions
No market analysis   No market analysis  Live data + RSI/MACD + BUY/SELL signals
No watch daemon      Heartbeat daemon    Full condition monitoring
No skill security    CVEs, Cisco found RCE  Security scan before any skill installs
```

**MICRODRAGON is not Claude Code's competitor. It is not OpenClaw's rival.**
**It is in a category of its own. The Universal Human-Level AI Agent.**

---

*© 2026 EMEMZYVISUALS DIGITALS — Emmanuel Ariyo*
*Questions & feedback → [@ememzyvisuals on X](https://x.com/ememzyvisuals)*
