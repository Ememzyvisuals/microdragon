# MICRODRAGON — Future Features & What Needs Adding

This document is an honest assessment of what MICRODRAGON has, what it needs, and where it
can build a decisive competitive advantage over OpenClaw and other agents.

---

## Current State (v0.1.0)

### ✅ Complete
- Rust core engine (tokio, async task orchestration)
- ModelRouter (Anthropic, OpenAI, Groq, OpenRouter, Custom/Ollama)
- Intent parser + multi-step task planner
- Full CLI with clap (all major commands)
- Cross-platform terminal detection (Windows CMD → Rich terminals)
- Animation system: spinners, progress bars, status transitions, typewriter
- Real-time streaming output renderer
- Interactive REPL shell with rustyline (history, editing, completions)
- First-run setup wizard
- WhatsApp bridge (whatsapp-web.js, QR auth, context memory, rate limiting)
- Telegram bot (long-poll, slash commands, streaming, context per user)
- Discord bot (gateway WebSocket, prefix commands)
- Security: AES-256-GCM encryption, prompt injection guard, sandbox, audit log
- Capability-scoped permissions model
- Coding module (Python): gen, debug, review, test, git integration
- Research module (Python): DuckDuckGo search, async crawl, HTML extraction
- Business module (Python): Yahoo Finance, RSI/MACD/trend, trading signals
- Simulation module (Python): PyAutoGUI, OpenCV, screen capture, app control
- Design module (Python): HTML/CSS gen, SVG, color palettes, Tailwind
- Automation engine: Playwright browser + PyAutoGUI desktop
- SQLite persistence schema (messages, tasks, code artifacts, settings)
- Shared types + integration test suite
- Full README + example config

---

## 🔴 Critical Missing (Must Build Next)

### 1. Memory Persistence → SQLite (HIGHEST PRIORITY)
**Current state:** Conversations live in RAM and disappear on restart.
**What's needed:**
- Wire `persistence/src/mod.rs` (Database struct) into `MemoryStore`
- Load last N messages on startup per session
- Auto-save every exchange
- Session IDs so multiple contexts can coexist
- `microdragon tasks history` should query the real DB, not in-memory

**Impact:** Without this, MICRODRAGON forgets everything. OpenClaw persists to Markdown files.

---

### 2. HTTP API Server for Module IPC (HIGH PRIORITY)
**Current state:** Modules are stubs — the Rust engine can't actually call Python/Node.
**What's needed:**
- Axum HTTP server on `127.0.0.1:7700` inside the Rust core
- POST `/api/chat` → processes prompt, returns JSON (used by social bots NOW)
- POST `/api/module/{name}` → dispatches to Python subprocess
- Python modules: simple aiohttp server on a port, or stdin/stdout JSON-RPC
- The social bots (WhatsApp, Telegram, Discord) already call `localhost:7700/api/chat` — it needs to exist

**Impact:** Social bots are written and ready but have nothing to call. This makes them live.

---

### 3. Voice Input/Output
**What's needed:**
- Whisper (OpenAI or local) for speech-to-text
- ElevenLabs or system TTS for text-to-speech
- macOS: use `say` command as free fallback
- Windows: use SAPI
- Wake word detection (Porcupine or Vosk)
- `microdragon voice` CLI mode

**Competitive edge:** OpenClaw has voice on macOS/iOS/Android. MICRODRAGON should match this.

---

### 4. Vector Database (Local Semantic Memory)
**What's needed:**
- Embed every conversation turn with a local embedding model (nomic-embed-text via Ollama, or API)
- Store vectors in a local DB (Qdrant embedded, or pure Rust `hora` / `usearch`)
- On each query: semantic search over past context (not just last N messages)
- Enables: "What did I tell you about project X 3 weeks ago?"

**Competitive edge over OpenClaw:** OpenClaw uses flat Markdown files. MICRODRAGON with vector search is far smarter.

---

### 5. Skill/Plugin System
**What's needed:**
- Define `SKILL.md` format (compatible with OpenClaw's ClawHub format for portability)
- Local skill directory: `~/.local/share/microdragon/skills/`
- `microdragon skill install <url>` — fetch and validate skill
- `microdragon skill list / remove`
- Brain auto-selects relevant skills per task
- Security: skill sandboxing, no arbitrary code execution without user confirmation

**Competitive edge:** OpenClaw's ClawHub has hundreds of community skills but NO security vetting (Cisco found RCE). MICRODRAGON's skill system should require signature/review.

---

### 6. Multi-Agent Orchestration
**What's needed:**
- Ability to spin up multiple named agents with different configs
- Agent roles: researcher, coder, writer, reviewer
- Agents communicate via shared message queue
- `microdragon agents create researcher --provider groq --model llama3`
- `microdragon agents run "research X then write Y"` → auto-routes to correct agent

**Why:** OpenClaw supports isolated agent workspaces. MICRODRAGON should beat this with typed roles.

---

### 7. Windows Native Installer
**What's needed:**
- `install.ps1` PowerShell script
- NSIS or WiX installer `.exe`
- Auto-detect Windows Terminal vs CMD
- Register `microdragon` on PATH automatically

---

### 8. Email Integration
**What's needed:**
- IMAP/SMTP client (lettre crate in Rust, or Python imaplib)
- Read inbox, reply, compose
- `microdragon email summary` → summarize unread
- `microdragon email draft "reply to john about meeting"` → AI draft, confirm before send
- Secure: never auto-send without explicit confirmation

**Competitive edge:** OpenClaw can delete your entire Gmail (documented incident). MICRODRAGON should require confirmation on every email action.

---

## 🟡 Important Improvements

### 9. Research Module: Real Search API
- Current: DuckDuckGo HTML scraping (fragile)
- Needed: Brave Search API (free tier), SerpAPI, or Tavily
- Add `microdragon config set-key brave-search <key>`

### 10. Code Execution Environment
- Docker container per code run (full isolation)
- Support: Python, Node.js, Rust, Go, Bash
- Stream stdout in real-time
- Time + memory limits
- `microdragon code run file.py`

### 11. File Manager
- `microdragon files read path/to/file` → AI reads and summarizes
- `microdragon files find "files related to X"` → semantic file search
- `microdragon files create "write a poem" --output poem.txt`

### 12. Calendar & Task Management
- Read/write system calendar (CalDAV or macOS/Google Calendar)
- `microdragon calendar today` → today's schedule + AI brief
- `microdragon tasks add "Review PR by 5pm"` → local task queue

### 13. Streaming Persistence
- Write partial responses to DB as they stream (crash recovery)
- Resume interrupted long responses

### 14. Rate Limit Recovery
- Exponential backoff is implemented but needs provider-specific retry-after parsing
- Groq rate limits are aggressive — need queuing

### 15. Config Hot-Reload
- Currently requires restart to pick up config changes
- Watch config file with `notify` crate

---

## 🟢 Competitive Differentiators to Build

### 16. Security Audit Dashboard
```
microdragon security audit
```
Shows:
- Last 50 audit log entries
- Blocked prompt injections
- Sandboxed command attempts
- Data access summary

This is a direct jab at OpenClaw's security reputation.

### 17. Offline Mode
- Cache last N AI responses
- When API is down: serve cached + explain "offline mode"
- Local model fallback (if Ollama configured)

### 18. MICRODRAGON Watch (Background Daemon)
```
microdragon watch start
```
- Runs MICRODRAGON in background on a heartbeat (like OpenClaw)
- Checks for scheduled tasks, monitors conditions
- Push notifications to connected social platforms
- `microdragon watch "alert me when AAPL drops below $150"`

### 19. Conversation Export/Import
- `microdragon export --format markdown history.md`
- `microdragon export --format json full_backup.json`
- `microdragon import backup.json` — restore history on new machine

### 20. Web UI (Optional, Local)
- Simple React/Svelte SPA served from Rust (axum)
- `microdragon web` → opens browser to `http://localhost:7701`
- Chat interface, history browser, config panel
- Strictly local — no cloud

### 21. GitHub Integration
- `microdragon code pr-review <url>` → AI reviews a GitHub PR
- `microdragon code create-issue "bug: X crashes when Y"`
- Uses GitHub CLI or direct API

### 22. Document Intelligence
- Upload PDF/DOCX → MICRODRAGON reads and answers questions
- `microdragon read contract.pdf "what are the payment terms?"`
- `microdragon read paper.pdf --summarize`

---

## Summary: The 5 Most Important Things to Build

In priority order:

1. **HTTP API server** — makes the social bots live (1-2 days of work)
2. **SQLite memory persistence** — MICRODRAGON remembers things (1 day)
3. **Voice I/O** — matches OpenClaw's "magic" feel (2-3 days)
4. **Skill system** — extensibility with security (3-5 days)
5. **Multi-agent orchestration** — the future of agents (1 week)

Everything else is polish. These five close the gap with OpenClaw completely
and in several areas (security, memory, performance) MICRODRAGON already wins.
