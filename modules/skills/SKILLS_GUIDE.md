"""
microdragon/modules/skills/SKILLS_GUIDE.md
═══════════════════════════════════════════════════════════════════════════════

# MICRODRAGON SKILLS — Complete Guide

Skills are plugins that extend MICRODRAGON with new capabilities.
Any developer can write a skill in Python. No Rust knowledge needed.

---

## What Are Skills?

A skill is a Python module inside a folder that MICRODRAGON can:
1. Discover (by scanning skills/ directory or installing from URL)
2. Security-scan (automatic before install)
3. Load (import and register with the skill engine)
4. Execute (user says "microdragon skills run weather London")

Skills get access to MICRODRAGON's AI brain, memory, and tool registry.

---

## Types of Skills

| Type | What It Does | Example |
|---|---|---|
| **data** | Fetch/process data | Stock prices, weather, sports scores |
| **automation** | Control apps/browser | Form filler, screenshot taker |
| **ai** | Custom AI workflows | Code reviewer, content generator |
| **integration** | Connect external APIs | Slack poster, Notion updater |
| **research** | Web intelligence | Domain scanner, news aggregator |
| **utility** | Transform data | Unit converter, QR generator |

---

## Skill Structure

```
my_skill/
├── SKILL.md         ← metadata (required)
├── main.py          ← entry point (required)
├── requirements.txt ← dependencies (optional)
└── config.example   ← config template (optional)
```

### SKILL.md (required)

```markdown
# Skill: Weather
**Author:** your_github_username
**Version:** 1.0.0
**Type:** data
**Description:** Real-time weather for any city
**Commands:**
  - weather <city>          : current weather
  - weather forecast <city> : 5-day forecast
**Requirements:** requests>=2.28.0
**Permissions:** network
**License:** MIT
```

### main.py (required)

```python
from microdragon.skills import SkillBase, command, SkillResult

class WeatherSkill(SkillBase):
    name = "weather"
    version = "1.0.0"
    description = "Real-time weather information"

    @command("weather")
    async def get_weather(self, city: str, **kwargs) -> SkillResult:
        import requests
        try:
            # Free weather API — no key needed for basic
            url = f"https://wttr.in/{city}?format=j1"
            r = requests.get(url, timeout=5)
            data = r.json()
            current = data["current_condition"][0]
            temp_c = current["temp_C"]
            desc = current["weatherDesc"][0]["value"]
            feels = current["FeelsLikeC"]
            humidity = current["humidity"]
            wind_kmph = current["windspeedKmph"]

            return SkillResult(
                success=True,
                output=f"Weather in {city}: {temp_c}°C, {desc}\\n"
                       f"Feels like: {feels}°C | Humidity: {humidity}% | Wind: {wind_kmph} km/h"
            )
        except Exception as e:
            return SkillResult(success=False, error=str(e))

    @command("forecast")
    async def get_forecast(self, city: str, **kwargs) -> SkillResult:
        import requests
        try:
            url = f"https://wttr.in/{city}?format=%l:+%C+%t+%h+%w\\n"
            r = requests.get(url, timeout=5)
            return SkillResult(success=True, output=r.text)
        except Exception as e:
            return SkillResult(success=False, error=str(e))
```

---

## Installing a Skill

```bash
# From GitHub URL
microdragon skills install https://github.com/username/microdragon-weather

# From local directory
microdragon skills install ./my_skill/

# From MICRODRAGON Skill Registry (coming soon)
microdragon skills install weather
```

What happens during install:
1. MICRODRAGON downloads the skill files
2. **Security scan runs automatically:**
   - Checks for data exfiltration patterns (no requests.post to unknown URLs)
   - Checks for dangerous subprocess calls
   - Checks for eval()/exec() usage
   - Checks for file system access outside allowed paths
3. If scan passes → skill is installed
4. If scan fails → install rejected with reason

---

## Running Skills

```bash
microdragon skills run weather London
microdragon skills run weather forecast "New York"
microdragon skills list
microdragon skills remove weather
microdragon skills scan ./my_new_skill/   # manual security scan
```

From Simple Mode:
```
you → Check the weather in Lagos
MICRODRAGON → [routes to weather skill automatically if installed]
         Weather in Lagos: 32°C, Partly cloudy
```

---

## SkillBase API Reference

```python
class SkillBase:
    # Available to all skills:

    async def ask_ai(self, prompt: str) -> str:
        # Use MICRODRAGON's configured AI brain
        ...

    async def remember(self, key: str, value: str):
        # Store data in MICRODRAGON's memory
        ...

    async def recall(self, key: str) -> str:
        # Retrieve from memory
        ...

    async def browse(self, url: str) -> str:
        # Fetch webpage content
        ...

    async def notify(self, message: str, platform: str = "all"):
        # Send notification to configured platforms
        ...
```

---

## Permission System

Skills declare permissions in SKILL.md. MICRODRAGON requests user confirmation for:

| Permission | What It Allows |
|---|---|
| `network` | HTTP requests to external URLs |
| `filesystem` | Read/write files in allowed paths |
| `shell` | Run shell commands (high risk, rare) |
| `ai` | Use MICRODRAGON's AI brain |
| `memory` | Read/write MICRODRAGON memory |
| `social` | Post to social platforms |
| `email` | Send emails (always requires confirmation) |

Dangerous permissions (`shell`, `email`) require explicit confirmation from the user every time.

---

## Example Skills

### 1. QR Code Generator

```python
from microdragon.skills import SkillBase, command, SkillResult

class QRSkill(SkillBase):
    name = "qr"
    version = "1.0.0"

    @command("qr")
    async def make_qr(self, text: str, output: str = "qr.png", **kwargs) -> SkillResult:
        import qrcode
        img = qrcode.make(text)
        img.save(output)
        return SkillResult(success=True, output=f"QR code saved to {output}")
```

### 2. Crypto Price Tracker

```python
from microdragon.skills import SkillBase, command, SkillResult
import aiohttp

class CryptoSkill(SkillBase):
    name = "crypto"

    @command("price")
    async def get_price(self, symbol: str, **kwargs) -> SkillResult:
        async with aiohttp.ClientSession() as s:
            async with s.get(f"https://api.coingecko.com/api/v3/simple/price"
                             f"?ids={symbol}&vs_currencies=usd") as r:
                data = await r.json()
        if symbol in data:
            price = data[symbol]['usd']
            return SkillResult(success=True,
                               output=f"{symbol.upper()}: ${price:,.2f}")
        return SkillResult(success=False, error=f"Symbol not found: {symbol}")
```

### 3. AI Content Summariser

```python
from microdragon.skills import SkillBase, command, SkillResult

class SummarySkill(SkillBase):
    name = "summarise"

    @command("summarise")
    async def summarise(self, url: str, **kwargs) -> SkillResult:
        content = await self.browse(url)
        summary = await self.ask_ai(
            f"Summarise this in 3 bullet points:\\n\\n{content[:3000]}"
        )
        return SkillResult(success=True, output=summary)
```

### 4. Domain OSINT Scanner

```python
from microdragon.skills import SkillBase, command, SkillResult
import socket, subprocess

class OsintSkill(SkillBase):
    name = "osint"

    @command("domain")
    async def scan_domain(self, domain: str, **kwargs) -> SkillResult:
        results = []
        # IP resolution
        try:
            ip = socket.gethostbyname(domain)
            results.append(f"IP: {ip}")
        except: pass
        # WHOIS
        content = await self.browse(f"https://who.is/whois/{domain}")
        summary = await self.ask_ai(
            f"Extract: registrar, creation date, expiry, nameservers from:\\n{content[:2000]}"
        )
        results.append(summary)
        return SkillResult(success=True, output='\\n'.join(results))
```

---

## Contributing Skills to the Registry

1. Create your skill following the structure above
2. Add tests in `tests/test_skill.py`
3. Open a PR to: https://github.com/ememzyvisuals/microdragon-skills
4. Security team reviews (automated + manual)
5. Approved skills appear in `microdragon skills install <name>`

---

## Skill Development Tips

- **Always return SkillResult** — never raise exceptions to the caller
- **Handle network timeouts** — set timeout=5-10s on all HTTP calls
- **Test offline** — your skill should fail gracefully without internet
- **One command, one job** — small focused commands are better than large ones
- **Use self.ask_ai()** — delegate complex reasoning to MICRODRAGON's brain
- **Document edge cases** — in SKILL.md, explain what happens if city not found, etc.

---

*© 2026 EMEMZYVISUALS DIGITALS — MICRODRAGON Skills System*
"""
