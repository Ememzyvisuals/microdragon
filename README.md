# 🐉 MICRODRAGON

**Distributed Intelligence Network — Cognitive Local Autonomous Agent**

Built by **EMEMZYVISUALS DIGITALS** · Emmanuel Ariyo · v0.1.14

---

MICRODRAGON is a CLI-first AI agent built in Rust. Every query passes through a **9-phase agentic pipeline** — it doesn't just chat, it perceives, plans, gathers real data, reasons, reflects on its own output, refines it, and commits to memory.

```
microdragon ask "research the latest developments in quantum computing"
```

```
  🐉 Agentic Pipeline
  ▶ 1/9  PERCEIVE    intent=RESEARCH entities=2
  ▶ 2/9  PLAN        5 steps: Formulate queries → Search web → Synthesise...
  ▶ 3/9  GATHER      Searching web: "quantum computing 2026 latest"...
  ▶ 4/9  SYNTHESIZE  context_tokens~3200 gathered_bytes=4812
  ▶ 5/9  REASON      model=llama-3.3-70b tokens=1847 latency=2341ms
  ▶ 6/9  REFLECT     score=8.4/10 refine=false
  ▶ 7/9  REFINE      skipped — quality 8.4/10 sufficient
  ▶ 8/9  FORMAT      output_chars=3421
  ✓      Pipeline complete · groq · 4102ms · 2198 tokens

  ## Quantum Computing — Latest Developments (2026)
  ...
```

---

## Installation

```bash
npm install -g @ememzyvisuals/microdragon
microdragon setup
```

**Requirements:** Node.js 18+ · One API key (Groq is free at console.groq.com)

---

## Commands

### Ask
```bash
microdragon ask "explain RSA encryption"
microdragon ask "latest bitcoin price"
microdragon ask "review this file" ./main.rs
```

### Chat (interactive)
```bash
microdragon
microdragon chat
```
In-session: `/clear` `/memory` `/status` `/exit`

### Code
```bash
microdragon code generate "REST API with auth in Python"
microdragon code generate "binary search tree" --language rust --output bst.rs
microdragon code debug ./src/main.rs
microdragon code review ./src/engine.rs
microdragon code test ./src/
microdragon code git "squash my last 3 commits"
```

### Research (real web search)
```bash
microdragon research "AI safety approaches in 2026"
microdragon research "Rust vs Go systems programming" --sources 8
microdragon research "history of Yoruba empire" --output report.md
```

### Business / Market
```bash
microdragon business market BTC
microdragon business market AAPL --interval 4h
microdragon business portfolio
microdragon business risk NVDA
```

### Automate (generates scripts)
```bash
microdragon automate browser "scrape product prices from amazon.com"
microdragon automate desktop "open Notepad and type a daily journal entry"
```

### Config & Status
```bash
microdragon setup               # First-time wizard
microdragon status              # Health check
microdragon config show         # Print config
microdragon config provider groq
microdragon config reset
microdragon --version
```

---

## AI Providers

| Provider | Free | Best Model | Speed |
|---|---|---|---|
| **Groq** ⭐ | ✅ Yes | llama-3.3-70b-versatile | ⚡ Very fast |
| **OpenAI** | ❌ Paid | gpt-4o | 🔵 Fast |
| **Anthropic** | ❌ Paid | claude-sonnet-4-5 | 🔵 Fast |
| **OpenRouter** | ✅ Free tier | many | Variable |
| **Ollama** | ✅ Local/free | llama3.1 | Depends on hardware |

For fully offline use:
```bash
# Install from ollama.ai, then:
ollama pull llama3.1 && ollama serve
microdragon config provider ollama
```

---

## Optional: Real Web Search

Research uses DuckDuckGo by default (no key needed). For higher quality:
```bash
# Get free key at api.search.brave.com
export BRAVE_SEARCH_API_KEY=your_key_here
```

---

## Optional: Python Automation

Only needed for `microdragon automate` script execution:
```bash
pip install playwright pyautogui
playwright install chromium
```

---

## The 9-Phase Pipeline

| # | Phase | What it does |
|---|---|---|
| 1 | **PERCEIVE** | Parse intent, classify task, extract entities |
| 2 | **PLAN** | Build task-specific execution strategy |
| 3 | **GATHER** | Run tools: web search, file read, memory recall |
| 4 | **SYNTHESIZE** | Merge all data into enriched context |
| 5 | **REASON** | Primary AI generation with full context |
| 6 | **REFLECT** | AI critiques own output (completeness, accuracy, structure) |
| 7 | **REFINE** | Re-generates if reflection score < 7/10 (skipped otherwise) |
| 8 | **FORMAT** | Apply markdown, headers, code blocks, citations |
| 9 | **COMMIT** | Persist to SQLite memory, emit telemetry |

---

## Build from Source

```bash
git clone https://github.com/Ememzyvisuals/microdragon
cd microdragon/core
cargo build --release
./target/release/microdragon setup
```

Requires Rust 1.75+. Pure Rust TLS — no OpenSSL required.

---

## Project Structure

```
microdragon/
├── core/src/
│   ├── engine/pipeline.rs   ← 9-phase agentic pipeline
│   ├── brain/               ← AI model router, intent parser, planner
│   ├── tools/               ← Web search (Brave/DDG), file reader
│   ├── memory/              ← SQLite conversation memory
│   ├── cli/                 ← CLI commands, markdown renderer
│   └── security/            ← Encryption, audit log
├── modules/social/          ← WhatsApp/Telegram bridges
└── npm/                     ← npm package wrapper
```

---

## License

MIT © 2026 EMEMZYVISUALS DIGITALS — Emmanuel Ariyo

*Final year project: Rust autonomous AI agent with 9-phase reasoning pipeline, multi-provider LLM routing, real web search, SQLite memory, and terminal markdown rendering.*
