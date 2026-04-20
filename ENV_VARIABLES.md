# Microdragon — Complete Environment Variables Reference
# ═══════════════════════════════════════════════════════════════════════════════
#
# HOW TO SET THEM:
#
# Linux/macOS — add to ~/.bashrc or ~/.zshrc:
#   export VARIABLE_NAME="your_value_here"
#   source ~/.bashrc
#
# Windows (PowerShell):
#   $env:VARIABLE_NAME = "your_value_here"
#   Or permanently: [System.Environment]::SetEnvironmentVariable("NAME","VALUE","User")
#
# Windows (Command Prompt):
#   set VARIABLE_NAME=your_value_here
#
# In a .env file (in your project directory):
#   VARIABLE_NAME=your_value_here
#   (Microdragon loads .env automatically if present)
#
# ═══════════════════════════════════════════════════════════════════════════════

# ─── AI PROVIDERS ─────────────────────────────────────────────────────────────

# Anthropic Claude (claude-opus-4-6, claude-sonnet-4-6, claude-haiku-4-5)
# Get key: https://console.anthropic.com → API Keys
ANTHROPIC_API_KEY=sk-ant-...

# OpenAI (gpt-4o, gpt-4o-mini, o1, o3)
# Get key: https://platform.openai.com → API Keys
OPENAI_API_KEY=sk-...

# Groq — FASTEST, free tier available (llama-3.3-70b, llama-3.1-8b-instant)
# Get key: https://console.groq.com → No credit card required
GROQ_API_KEY=gsk_...

# xAI Grok (grok-3, grok-3-mini, grok-2-1212)
# Get key: https://console.x.ai → API Keys
XAI_API_KEY=xai-...

# Google Gemini (gemini-2.5-flash, gemini-2.5-pro, gemini-2.0-flash)
# Get key: https://aistudio.google.com → Get API Key (free)
GEMINI_API_KEY=...

# OpenRouter — access 200+ models with one key
# Get key: https://openrouter.ai/keys
OPENROUTER_API_KEY=sk-or-...

# Cohere (command-r-plus, command-r)
# Get key: https://dashboard.cohere.com
COHERE_API_KEY=...

# Mistral AI (mistral-large, mistral-nemo)
# Get key: https://console.mistral.ai
MISTRAL_API_KEY=...

# Custom OpenAI-compatible endpoint (Ollama, vLLM, LM Studio, etc.)
# Set to your local or custom endpoint
MICRODRAGON_CUSTOM_ENDPOINT=http://localhost:11434/v1
MICRODRAGON_CUSTOM_MODEL=llama3.1
MICRODRAGON_CUSTOM_API_KEY=none   # "none" = no auth (Ollama)

# ─── ACTIVE PROVIDER SELECTION ────────────────────────────────────────────────
# Which provider to use. Options: anthropic | openai | groq | xai | gemini |
#                                 openrouter | cohere | mistral | custom
# Leave unset to auto-select based on available keys (prefers: groq > openai > anthropic)
MICRODRAGON_PROVIDER=groq

# Which model to use (provider-specific)
# Leave unset to use provider's recommended model
MICRODRAGON_MODEL=llama-3.3-70b-versatile

# ─── COST CONTROL ─────────────────────────────────────────────────────────────

# Hard daily budget cap in USD. Blocks calls when reached.
MICRODRAGON_DAILY_BUDGET_USD=1.00

# Prefer cheapest model when multiple are available
MICRODRAGON_PREFER_CHEAP=false

# Prefer local/Ollama model when available (zero cost)
MICRODRAGON_PREFER_LOCAL=false

# ─── SOCIAL PLATFORMS ─────────────────────────────────────────────────────────

# Telegram Bot
# Create: @BotFather → /newbot → copy token
TELEGRAM_BOT_TOKEN=1234567890:ABCdef...

# Discord Bot  
# Create: discord.com/developers → New App → Bot → Copy Token
DISCORD_BOT_TOKEN=...

# X (Twitter) API v2
# Create: developer.x.com → New Project → New App
X_BEARER_TOKEN=AAAA...          # Read-only public data
X_API_KEY=...                   # OAuth 1.0a consumer key
X_API_KEY_SECRET=...            # OAuth 1.0a consumer secret
X_ACCESS_TOKEN=...              # Your user access token
X_ACCESS_TOKEN_SECRET=...       # Your user access token secret
X_CLIENT_ID=...                 # OAuth 2.0 client ID (optional)
X_CLIENT_SECRET=...             # OAuth 2.0 client secret (optional)

# ─── MEDIA GENERATION ─────────────────────────────────────────────────────────

# HuggingFace — free tier, FLUX.1-dev image generation
# Get token: https://huggingface.co/settings/tokens (free)
HUGGINGFACE_TOKEN=hf_...

# Fal.ai — fast image/video generation
# Get key: https://fal.ai/dashboard
FAL_KEY=...

# Stability AI — Stable Image Ultra
# Get key: https://platform.stability.ai
STABILITY_API_KEY=sk-...

# ElevenLabs — best quality TTS
# Get key: https://elevenlabs.io/app/settings (free tier available)
ELEVENLABS_API_KEY=...
ELEVENLABS_VOICE_ID=21m00Tcm4TlvDq8ikWAM   # default: Rachel voice

# ─── GITHUB ───────────────────────────────────────────────────────────────────
# Get: github.com/settings/tokens → Generate new token (classic) → repo scope
GITHUB_TOKEN=ghp_...

# ─── EMAIL ────────────────────────────────────────────────────────────────────
# Gmail: use App Password (not regular password)
# Enable: myaccount.google.com → Security → 2FA → App Passwords
MICRODRAGON_EMAIL=you@gmail.com
MICRODRAGON_EMAIL_PASSWORD=your_16char_app_password

# Outlook/Hotmail
# MICRODRAGON_EMAIL=you@outlook.com
# MICRODRAGON_EMAIL_PASSWORD=your_password

# SMTP server (auto-detected from email domain, override if needed)
# MICRODRAGON_SMTP_HOST=smtp.gmail.com
# MICRODRAGON_SMTP_PORT=587

# ─── CALENDAR ─────────────────────────────────────────────────────────────────
# Google Calendar: download credentials.json from console.cloud.google.com
GOOGLE_CALENDAR_CREDENTIALS=/path/to/credentials.json

# ─── VOICE (STT / TTS) ────────────────────────────────────────────────────────
# STT is auto-selected based on AI provider. Override:
# Options: groq | openai | local (faster-whisper)
MICRODRAGON_STT_PROVIDER=groq   # or: openai, local

# Whisper model size for local STT (tiny=fastest, large=best quality)
MICRODRAGON_WHISPER_MODEL=base  # tiny | base | small | medium | large

# TTS depends on AI provider. ElevenLabs key overrides everything.
# macOS: uses 'say' command automatically
# Linux: uses piper if installed, else espeak
# Windows: uses SAPI automatically
MACOS_TTS_VOICE=Samantha        # macOS only: voice name

# ─── AWS / GOOGLE CLOUD / AZURE ───────────────────────────────────────────────
# AWS Bedrock (for AWS-hosted models)
AWS_ACCESS_KEY_ID=...
AWS_SECRET_ACCESS_KEY=...
AWS_DEFAULT_REGION=us-east-1

# Google Cloud / Vertex AI
GOOGLE_APPLICATION_CREDENTIALS=/path/to/service-account.json
GOOGLE_CLOUD_PROJECT=your-project-id

# Azure OpenAI
AZURE_OPENAI_API_KEY=...
AZURE_OPENAI_ENDPOINT=https://your-resource.openai.azure.com
AZURE_OPENAI_DEPLOYMENT=gpt-4o

# ─── MCP SERVERS ──────────────────────────────────────────────────────────────
# PostgreSQL MCP server
POSTGRES_CONNECTION_STRING=postgresql://user:pass@localhost:5432/db

# Slack MCP server
SLACK_BOT_TOKEN=xoxb-...

# Notion MCP server
NOTION_API_KEY=secret_...

# ─── MICRODRAGON BEHAVIOUR ────────────────────────────────────────────────────
# Log level: trace | debug | info | warn | error
MICRODRAGON_LOG=info

# Data directory (default: ~/.local/share/microdragon)
MICRODRAGON_DATA_DIR=~/.local/share/microdragon

# Disable all color output (for CI or plain terminals)
NO_COLOR=1

# API server port (default: 7700)
MICRODRAGON_API_PORT=7700

# Daemon mode port
MICRODRAGON_DAEMON_PORT=7701

# Personal assistant persona
# Options: developer | ceo | trader | gamer | business | student | generic
MICRODRAGON_PERSONA=developer

# Your name (used in PA briefings and greetings)
MICRODRAGON_USER_NAME=Emmanuel

# ─── HARNESS SETTINGS ─────────────────────────────────────────────────────────
# Enable/disable the Dragon Harness model amplification
MICRODRAGON_HARNESS_ENABLED=true

# Anti-drift sensitivity (0.0 = off, 1.0 = maximum)
MICRODRAGON_ANTI_DRIFT=0.6

# ─── QUICK START (minimum required) ──────────────────────────────────────────
#
# OPTION A — Groq (free, no credit card, fastest):
#   export GROQ_API_KEY="gsk_..."        # from console.groq.com
#   microdragon setup
#   microdragon
#
# OPTION B — Anthropic:
#   export ANTHROPIC_API_KEY="sk-ant-..." # from console.anthropic.com
#   microdragon setup
#   microdragon
#
# OPTION C — 100% free, 100% offline (Ollama):
#   ollama pull llama3.1
#   export MICRODRAGON_CUSTOM_ENDPOINT="http://localhost:11434/v1"
#   export MICRODRAGON_CUSTOM_MODEL="llama3.1"
#   export MICRODRAGON_PROVIDER="custom"
#   microdragon
#
# ─────────────────────────────────────────────────────────────────────────────
# 🔒 SECURITY: Keys are stored encrypted (AES-256-GCM) at:
#    ~/.local/share/microdragon/keys.enc
#    They are NEVER transmitted to EMEMZYVISUALS DIGITALS or any third party.
#    They only go to the provider API you configure.
# ─────────────────────────────────────────────────────────────────────────────
