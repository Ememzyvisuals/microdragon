"""
microdragon/modules/social/src/telegram/bot.py
MICRODRAGON Telegram Bot — Full CLI parity via Telegram
"""

import asyncio
import logging
import json
import aiohttp
import os
from typing import Optional

logger = logging.getLogger("microdragon.telegram")

# ─── Config ───────────────────────────────────────────────────────────────────

MICRODRAGON_CORE_URL = os.getenv("MICRODRAGON_CORE_URL", "http://127.0.0.1:7700")
BOT_TOKEN = os.getenv("TELEGRAM_BOT_TOKEN", "")
ALLOWED_USER_IDS = [
    int(x) for x in os.getenv("TELEGRAM_ALLOWED_USERS", "").split(",") if x.strip()
]
RATE_LIMIT_SECS = float(os.getenv("MICRODRAGON_RATE_LIMIT_SECS", "2"))

# ─── Context store ────────────────────────────────────────────────────────────

context_store: dict[int, list] = {}
last_message_time: dict[int, float] = {}

def get_context(user_id: int) -> list:
    if user_id not in context_store:
        context_store[user_id] = []
    return context_store[user_id]

def add_to_context(user_id: int, role: str, content: str):
    ctx = get_context(user_id)
    ctx.append({"role": role, "content": content})
    while len(ctx) > 40:
        ctx.pop(0)

def clear_context(user_id: int):
    context_store[user_id] = []

# ─── Microdragon Core client ─────────────────────────────────────────────────────────

async def call_microdragon_core(input_text: str, context: list, source: str = "telegram") -> str:
    payload = {"input": input_text, "context": context, "source": source}
    try:
        async with aiohttp.ClientSession() as session:
            async with session.post(
                f"{MICRODRAGON_CORE_URL}/api/chat",
                json=payload,
                timeout=aiohttp.ClientTimeout(total=90),
                headers={"X-MICRODRAGON-Source": source}
            ) as resp:
                data = await resp.json()
                return data.get("response", "No response from MICRODRAGON.")
    except asyncio.TimeoutError:
        return "⏱ MICRODRAGON timed out. Your task may still be running."
    except Exception as e:
        logger.error(f"Core call error: {e}")
        return f"⚠ MICRODRAGON core unavailable: {e}"

# ─── Telegram API helpers ─────────────────────────────────────────────────────

TELEGRAM_API = f"https://api.telegram.org/bot{BOT_TOKEN}"

async def tg_post(method: str, data: dict) -> dict:
    async with aiohttp.ClientSession() as session:
        async with session.post(f"{TELEGRAM_API}/{method}", json=data) as resp:
            return await resp.json()

async def send_message(chat_id: int, text: str, parse_mode: str = "Markdown"):
    # Split long messages (Telegram limit = 4096 chars)
    chunks = [text[i:i+4000] for i in range(0, len(text), 4000)]
    for chunk in chunks:
        await tg_post("sendMessage", {
            "chat_id": chat_id,
            "text": chunk,
            "parse_mode": parse_mode,
        })

async def send_typing(chat_id: int):
    await tg_post("sendChatAction", {"chat_id": chat_id, "action": "typing"})

# ─── Authorization ────────────────────────────────────────────────────────────

def is_authorized(user_id: int) -> bool:
    if not ALLOWED_USER_IDS:
        return True  # No restriction configured
    return user_id in ALLOWED_USER_IDS

# ─── Command handlers ─────────────────────────────────────────────────────────

async def handle_start(chat_id: int, user_id: int):
    await send_message(chat_id,
        "🤖 *MICRODRAGON AI Agent*\n\n"
        "I'm your personal AI assistant. Send me any task or question.\n\n"
        "Commands:\n"
        "`/help` — show commands\n"
        "`/status` — check status\n"
        "`/clear` — clear conversation\n"
        "`/code <task>` — code generation\n"
        "`/research <query>` — web research\n"
        "`/automate <task>` — automation script\n\n"
        "Or just send me any message!"
    )

async def handle_help(chat_id: int):
    await send_message(chat_id,
        "*MICRODRAGON Commands*\n\n"
        "`/start` — welcome message\n"
        "`/help` — this message\n"
        "`/status` — MICRODRAGON status\n"
        "`/clear` — clear your history\n"
        "`/code <description>` — generate code\n"
        "`/research <topic>` — research a topic\n"
        "`/automate <task>` — create automation\n"
        "`/business <query>` — market analysis\n\n"
        "Any other message → direct AI chat"
    )

async def handle_status(chat_id: int, user_id: int):
    ctx_len = len(get_context(user_id))
    await send_message(chat_id,
        f"*MICRODRAGON Status*\n"
        f"• Core: ✅ online\n"
        f"• Source: Telegram\n"
        f"• Your context: {ctx_len} messages\n"
        f"• Core URL: `{MICRODRAGON_CORE_URL}`"
    )

# ─── Message router ───────────────────────────────────────────────────────────

async def handle_update(update: dict):
    msg = update.get("message") or update.get("edited_message")
    if not msg:
        return

    chat_id = msg["chat"]["id"]
    user_id = msg["from"]["id"]
    text = msg.get("text", "").strip()

    if not text:
        return

    if not is_authorized(user_id):
        await send_message(chat_id, "⛔ Unauthorized. Contact the MICRODRAGON owner to gain access.")
        return

    # Rate limiting
    import time
    now = time.time()
    last = last_message_time.get(user_id, 0)
    if now - last < RATE_LIMIT_SECS:
        return
    last_message_time[user_id] = now

    # Route commands
    if text.startswith("/start"):
        await handle_start(chat_id, user_id)
        return
    if text.startswith("/help"):
        await handle_help(chat_id)
        return
    if text.startswith("/status"):
        await handle_status(chat_id, user_id)
        return
    if text.startswith("/clear"):
        clear_context(user_id)
        await send_message(chat_id, "✅ Conversation history cleared.")
        return

    # Build AI prompt from commands or direct text
    if text.startswith("/code "):
        prompt = f"Write complete, production-ready code for: {text[6:]}"
    elif text.startswith("/research "):
        prompt = f"Research thoroughly and provide a structured report on: {text[10:]}"
    elif text.startswith("/automate "):
        prompt = f"Create an automation script for: {text[10:]}"
    elif text.startswith("/business "):
        prompt = f"Market analysis for: {text[10:]}"
    else:
        prompt = text

    # Send typing indicator
    await send_typing(chat_id)

    # Add to context and call core
    add_to_context(user_id, "user", prompt)
    response = await call_microdragon_core(prompt, get_context(user_id))
    add_to_context(user_id, "assistant", response)

    await send_message(chat_id, response)

# ─── Long polling loop ────────────────────────────────────────────────────────

async def run_bot():
    if not BOT_TOKEN:
        logger.error("TELEGRAM_BOT_TOKEN not set. Run: microdragon config set-key telegram <token>")
        return

    logger.info("MICRODRAGON Telegram bot starting...")
    offset = 0

    async with aiohttp.ClientSession() as session:
        # Get bot info
        me_resp = await session.get(f"{TELEGRAM_API}/getMe")
        me = (await me_resp.json()).get("result", {})
        logger.info(f"Bot: @{me.get('username')} ({me.get('first_name')})")
        print(f"✓ MICRODRAGON Telegram bot @{me.get('username')} is live!")

        while True:
            try:
                resp = await session.get(
                    f"{TELEGRAM_API}/getUpdates",
                    params={"offset": offset, "timeout": 30, "allowed_updates": ["message"]},
                    timeout=aiohttp.ClientTimeout(total=35)
                )
                data = await resp.json()

                if data.get("ok"):
                    for update in data.get("result", []):
                        offset = update["update_id"] + 1
                        asyncio.create_task(handle_update(update))

            except asyncio.CancelledError:
                break
            except Exception as e:
                logger.error(f"Polling error: {e}")
                await asyncio.sleep(5)


if __name__ == "__main__":
    logging.basicConfig(level=logging.INFO)
    asyncio.run(run_bot())
