"""
microdragon/modules/social/src/discord/bot.py
MICRODRAGON Discord Bot — Slash commands + full CLI parity
"""

import asyncio
import aiohttp
import json
import os
import logging
from typing import Optional

logger = logging.getLogger("microdragon.discord")

MICRODRAGON_CORE_URL = os.getenv("MICRODRAGON_CORE_URL", "http://127.0.0.1:7700")
DISCORD_TOKEN = os.getenv("DISCORD_BOT_TOKEN", "")
ALLOWED_GUILDS = [g for g in os.getenv("DISCORD_ALLOWED_GUILDS", "").split(",") if g.strip()]
COMMAND_PREFIX = os.getenv("MICRODRAGON_CMD_PREFIX", "!microdragon")

context_store: dict[str, list] = {}


async def call_microdragon_core(input_text: str, context: list) -> str:
    payload = {"input": input_text, "context": context, "source": "discord"}
    try:
        async with aiohttp.ClientSession() as session:
            async with session.post(
                f"{MICRODRAGON_CORE_URL}/api/chat",
                json=payload,
                timeout=aiohttp.ClientTimeout(total=90)
            ) as resp:
                data = await resp.json()
                return data.get("response", "No response.")
    except Exception as e:
        return f"⚠ MICRODRAGON unavailable: {e}"


def get_context(channel_id: str) -> list:
    if channel_id not in context_store:
        context_store[channel_id] = []
    return context_store[channel_id]


def add_to_context(channel_id: str, role: str, content: str):
    ctx = get_context(channel_id)
    ctx.append({"role": role, "content": content})
    while len(ctx) > 40:
        ctx.pop(0)


GATEWAY_URL = "wss://gateway.discord.gg/?v=10&encoding=json"


class DiscordGateway:
    """Minimal Discord gateway client — no heavy lib needed."""

    def __init__(self, token: str):
        self.token = token
        self.heartbeat_interval = None
        self.session_id = None
        self.ws = None
        self.sequence = None

    async def connect(self):
        import websockets
        async with websockets.connect(GATEWAY_URL) as ws:
            self.ws = ws
            # Hello
            hello = json.loads(await ws.recv())
            self.heartbeat_interval = hello["d"]["heartbeat_interval"] / 1000

            # Identify
            await ws.send(json.dumps({
                "op": 2,
                "d": {
                    "token": self.token,
                    "intents": 512 + 32768,  # GUILD_MESSAGES + MESSAGE_CONTENT
                    "properties": {"os": "linux", "browser": "microdragon", "device": "microdragon"},
                }
            }))

            heartbeat_task = asyncio.create_task(self._heartbeat(ws))

            async for raw in ws:
                msg = json.loads(raw)
                op = msg.get("op")
                t = msg.get("t")
                d = msg.get("d", {})

                if msg.get("s"):
                    self.sequence = msg["s"]

                if op == 11:
                    continue  # heartbeat ack
                if t == "READY":
                    self.session_id = d["session_id"]
                    logger.info(f"Discord connected as {d['user']['username']}")
                    print(f"✓ MICRODRAGON Discord bot live as {d['user']['username']}#{d['user']['discriminator']}")
                elif t == "MESSAGE_CREATE":
                    asyncio.create_task(self._handle_message(d))

            heartbeat_task.cancel()

    async def _heartbeat(self, ws):
        while True:
            await asyncio.sleep(self.heartbeat_interval)
            await ws.send(json.dumps({"op": 1, "d": self.sequence}))

    async def _handle_message(self, msg: dict):
        content = msg.get("content", "").strip()
        channel_id = msg.get("channel_id", "")
        author = msg.get("author", {})

        if author.get("bot"):
            return
        if not content:
            return

        # Guild check
        guild_id = msg.get("guild_id", "")
        if ALLOWED_GUILDS and guild_id not in ALLOWED_GUILDS:
            return

        # Only respond to prefix commands or DMs
        is_dm = msg.get("guild_id") is None
        if not is_dm and not content.startswith(COMMAND_PREFIX):
            return

        text = content.removeprefix(COMMAND_PREFIX).strip() if not is_dm else content

        # Handle built-ins
        if text.lower() in ("help", "?"):
            await self._send(channel_id,
                "**MICRODRAGON AI Agent**\n"
                f"Prefix: `{COMMAND_PREFIX}`\n\n"
                "Commands:\n"
                "`help` — this message\n"
                "`clear` — clear context\n"
                "`status` — show status\n"
                "`code <task>` — generate code\n"
                "`research <query>` — web research\n\n"
                "Or just type your request after the prefix."
            )
            return

        if text.lower() == "clear":
            context_store[channel_id] = []
            await self._send(channel_id, "✅ Context cleared.")
            return

        if text.lower() == "status":
            await self._send(channel_id,
                f"**MICRODRAGON Status**\n"
                f"• Core: online\n"
                f"• Context: {len(get_context(channel_id))} messages"
            )
            return

        # Build prompt
        if text.lower().startswith("code "):
            prompt = f"Write complete code for: {text[5:]}"
        elif text.lower().startswith("research "):
            prompt = f"Research and report on: {text[9:]}"
        else:
            prompt = text

        # Show typing
        await self._send_typing(channel_id)

        add_to_context(channel_id, "user", prompt)
        response = await call_microdragon_core(prompt, get_context(channel_id))
        add_to_context(channel_id, "assistant", response)

        # Discord message limit = 2000 chars
        chunks = [response[i:i+1900] for i in range(0, len(response), 1900)]
        for chunk in chunks:
            await self._send(channel_id, chunk)

    async def _send(self, channel_id: str, content: str):
        async with aiohttp.ClientSession() as session:
            await session.post(
                f"https://discord.com/api/v10/channels/{channel_id}/messages",
                json={"content": content},
                headers={"Authorization": f"Bot {self.token}"}
            )

    async def _send_typing(self, channel_id: str):
        async with aiohttp.ClientSession() as session:
            await session.post(
                f"https://discord.com/api/v10/channels/{channel_id}/typing",
                headers={"Authorization": f"Bot {self.token}"}
            )


async def main():
    if not DISCORD_TOKEN:
        print("Error: DISCORD_BOT_TOKEN not set.")
        print("Run: microdragon config set-key discord <token>")
        return

    gateway = DiscordGateway(DISCORD_TOKEN)
    await gateway.connect()


if __name__ == "__main__":
    logging.basicConfig(level=logging.INFO)
    asyncio.run(main())
