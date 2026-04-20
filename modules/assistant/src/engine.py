"""
microdragon/modules/assistant/src/engine.py
═══════════════════════════════════════════════════════════════════════════════
MICRODRAGON PERSONAL ASSISTANT MODULE
═══════════════════════════════════════════════════════════════════════════════

MICRODRAGON acts as your personal assistant at any level:
  - Developer PA: standup prep, PR reviews, deployment checks
  - CEO PA: board meeting prep, KPI briefings, investor updates
  - Trader PA: market briefings, portfolio monitoring, signal alerts
  - Gamer PA: tournament prep, patch notes, strategy guides
  - Business Owner PA: sales briefings, customer follow-ups, invoices

© 2026 EMEMZYVISUALS DIGITALS — Emmanuel Ariyo
"""

import asyncio
import os
from dataclasses import dataclass, field
from typing import Optional
from datetime import datetime, timedelta
from enum import Enum


class PersonaType(Enum):
    DEVELOPER  = "developer"
    CEO        = "ceo"
    TRADER     = "trader"
    GAMER      = "gamer"
    BUSINESS   = "business_owner"
    STUDENT    = "student"
    RESEARCHER = "researcher"
    CREATOR    = "creator"
    GENERIC    = "generic"


@dataclass
class MeetingRequest:
    title: str
    attendees: list[str]
    duration_minutes: int
    preferred_times: list[str]
    location: str = "Video call"
    agenda: str = ""
    notes: str = ""


@dataclass
class DailyBriefing:
    persona: str
    date: str
    items: list[dict] = field(default_factory=list)
    priorities: list[str] = field(default_factory=list)
    alerts: list[str] = field(default_factory=list)
    schedule: list[dict] = field(default_factory=list)


class PersonalAssistantEngine:
    """
    MICRODRAGON Personal Assistant — adapts to any professional persona.
    Plans your day, books meetings, prepares briefings, remembers everything.
    """

    def __init__(self):
        self.persona = PersonaType.GENERIC
        self.user_name = ""
        self.timezone = "UTC"
        self.work_hours = (9, 18)  # 9am-6pm

    def set_persona(self, persona: str):
        """Set the assistant's operating persona."""
        mapping = {
            "developer": PersonaType.DEVELOPER,
            "dev": PersonaType.DEVELOPER,
            "ceo": PersonaType.CEO,
            "executive": PersonaType.CEO,
            "founder": PersonaType.CEO,
            "trader": PersonaType.TRADER,
            "investor": PersonaType.TRADER,
            "gamer": PersonaType.GAMER,
            "player": PersonaType.GAMER,
            "business": PersonaType.BUSINESS,
            "owner": PersonaType.BUSINESS,
            "student": PersonaType.STUDENT,
            "researcher": PersonaType.RESEARCHER,
            "creator": PersonaType.CREATOR,
        }
        self.persona = mapping.get(persona.lower(), PersonaType.GENERIC)

    def build_briefing_prompt(self, context: dict) -> str:
        """Build the morning briefing prompt for the configured persona."""
        today = datetime.now().strftime("%A, %B %d %Y")
        persona = self.persona.value

        prompts = {
            PersonaType.DEVELOPER: f"""You are MICRODRAGON acting as a senior developer personal assistant.
Today is {today}.

Prepare a developer morning briefing using this context:
{context}

Cover exactly:
1. TODAY'S FOCUS: What to build/fix today (1-3 priority tasks)
2. PRs TO REVIEW: Any pull requests awaiting review
3. CI/CD STATUS: Any failing builds or deployments
4. BLOCKERS: Any issues blocking progress
5. TECH NEWS: 2-3 relevant tech developments from today
6. SCHEDULE: Meetings/standups today

Format: Clean, concise. Developers hate fluff.""",

            PersonaType.CEO: f"""You are MICRODRAGON acting as an executive personal assistant to a CEO.
Today is {today}.

Prepare a CEO morning briefing using this context:
{context}

Cover exactly:
1. EXECUTIVE SUMMARY: 3 most critical business updates
2. KEY METRICS: Revenue, growth, churn vs targets
3. DECISIONS NEEDED: Items requiring CEO sign-off today
4. MEETINGS: Today's schedule with context for each
5. TEAM UPDATES: Key reports and their status
6. MARKET INTEL: Competitor moves, industry news
7. INVESTOR/BOARD: Any items requiring attention

Format: Concise, actionable. CEOs need signals not noise.""",

            PersonaType.TRADER: f"""You are MICRODRAGON acting as a personal assistant to an active trader.
Today is {today}.

Prepare a pre-market trading briefing using this context:
{context}

Cover exactly:
1. OVERNIGHT MOVES: Key price changes while you slept
2. MACRO: Fed, economic data, earnings today
3. WATCHLIST: Status of tracked tickers
4. SIGNALS: BUY/SELL/HOLD alerts on key positions
5. RISK: Any positions near stop-loss levels
6. CALENDAR: Today's earnings, data releases, FOMC events
7. SENTIMENT: Overall market mood (fear/greed index)

⚠ Always include: "Not financial advice."
Format: Fast, data-dense, signal-oriented.""",

            PersonaType.GAMER: f"""You are MICRODRAGON acting as a personal assistant to a competitive gamer.
Today is {today}.

Prepare a gaming session briefing using this context:
{context}

Cover exactly:
1. RANKED STATUS: Current rank, LP/MMR, streak
2. PATCH NOTES: Any recent changes affecting your main
3. META: Current tier list changes, strong picks
4. SESSION GOAL: Recommended focus for today's session
5. OPPONENTS: Any scouting on known opponents if tournament
6. TOURNAMENTS: Upcoming events you're registered for
7. CLIPS/VODs: Any notable plays from your recent sessions

Format: Hype but precise. Gamers need to focus.""",

            PersonaType.BUSINESS: f"""You are MICRODRAGON acting as a personal assistant to a business owner.
Today is {today}.

Prepare a business morning briefing using this context:
{context}

Cover exactly:
1. REVENUE TODAY: Sales so far, targets, pipeline
2. CUSTOMERS: Outstanding requests, complaints, renewals
3. TEAM: Any issues, absences, deliverables due
4. INVOICES: Outstanding payments, overdue accounts
5. OPERATIONS: Supplier updates, inventory, logistics
6. MARKETING: Campaign performance, social metrics
7. TODAY'S ACTIONS: 3 highest-priority tasks

Format: Practical, numbers-focused. Business owners need clarity.""",
        }

        return prompts.get(self.persona, f"""You are MICRODRAGON personal assistant.
Today is {today}. Context: {context}
Prepare a helpful daily briefing covering key priorities, schedule, and alerts.""")

    async def book_meeting(self, request: MeetingRequest,
                            calendar_module=None) -> dict:
        """Book a meeting with calendar integration and smart scheduling."""
        result = {
            "status": "draft",
            "meeting": {
                "title": request.title,
                "attendees": request.attendees,
                "duration": request.duration_minutes,
                "preferred_times": request.preferred_times,
                "location": request.location,
                "agenda": request.agenda,
            }
        }

        # Generate meeting invite content
        invite = self.format_meeting_invite(request)
        result["invite_text"] = invite

        # Book via calendar if available
        if calendar_module:
            try:
                booked = await calendar_module.create_event(
                    title=request.title,
                    attendees=request.attendees,
                    duration_minutes=request.duration_minutes,
                    preferred_time=request.preferred_times[0] if request.preferred_times else None,
                    location=request.location,
                    description=request.agenda,
                )
                result["status"] = "booked"
                result["calendar_event"] = booked
            except Exception as e:
                result["status"] = "draft"
                result["error"] = str(e)

        return result

    def format_meeting_invite(self, request: MeetingRequest) -> str:
        """Format a professional meeting invitation."""
        agenda_lines = ""
        if request.agenda:
            agenda_lines = f"\n\nAGENDA:\n{request.agenda}"

        return f"""Hi,

I hope you're well. I'd like to schedule a meeting:

  Title:     {request.title}
  Duration:  {request.duration_minutes} minutes
  Location:  {request.location}
  Attendees: {', '.join(request.attendees)}
{agenda_lines}

Could you confirm your availability at one of these times?
{chr(10).join(f'  • {t}' for t in request.preferred_times)}

A calendar invite will follow.

Best regards,
[Name]"""

    async def plan_day(self, date: str, tasks: list, meetings: list,
                        energy_level: str = "high") -> str:
        """Create an optimised daily plan."""
        energy_note = {
            "high": "Schedule deep work in the morning when energy is highest.",
            "medium": "Alternate focused work with lighter tasks.",
            "low": "Focus on meetings and admin. Save deep work for tomorrow."
        }.get(energy_level, "")

        time_blocks = []
        hour = self.work_hours[0]

        # Group by type
        deep_tasks = [t for t in tasks if t.get("type") == "deep_work"]
        admin_tasks = [t for t in tasks if t.get("type") == "admin"]
        comm_tasks  = [t for t in tasks if t.get("type") == "comms"]

        plan = f"""DAILY PLAN — {date}
{energy_note}

SCHEDULE:
"""
        # Add meetings to schedule
        for meeting in sorted(meetings, key=lambda m: m.get("time", "12:00")):
            plan += f"  {meeting['time']}  📅 {meeting['title']} ({meeting.get('duration', 30)}min)\n"

        plan += "\nPRIORITY TASKS:\n"
        for i, task in enumerate(tasks[:5], 1):
            plan += f"  {i}. {task.get('title', str(task))}\n"

        plan += f"\nPROTECTED FOCUS TIME: 09:00-12:00 (no meetings)"
        plan += f"\nEND OF DAY: Review tomorrow's plan at {self.work_hours[1]-1}:00"

        return plan

    async def prepare_meeting_notes(self, meeting_title: str,
                                     attendees: list, agenda: str) -> str:
        """Prepare pre-meeting briefing and a notes template."""
        return f"""PRE-MEETING BRIEF: {meeting_title}
{'─' * 50}
ATTENDEES: {', '.join(attendees)}
AGENDA: {agenda}

CONTEXT TO REVIEW:
  □ Last meeting notes (check calendar history)
  □ Open action items from previous meeting
  □ Relevant documents to bring

NOTES TEMPLATE:
{'─' * 50}
Date: {datetime.now().strftime('%Y-%m-%d %H:%M')}
Meeting: {meeting_title}
Present: {', '.join(attendees)}

DISCUSSION POINTS:
1.
2.
3.

DECISIONS MADE:
1.
2.

ACTION ITEMS:
  WHO               WHAT                          BY WHEN
  ──────────────────────────────────────────────────────
  
NEXT MEETING: [date/time TBD]
"""

    async def follow_up_email(self, meeting_title: str, attendees: list,
                               action_items: list) -> str:
        """Draft a follow-up email after a meeting."""
        items_formatted = '\n'.join(f"  • {item}" for item in action_items)
        return f"""Subject: Follow-up: {meeting_title}

Hi {', '.join(attendees)},

Thanks for the productive discussion today. Here's a summary of our agreed action items:

{items_formatted}

Please confirm if I've captured everything correctly. I'll follow up on progress by next week.

Best regards,
[Name]"""

    def build_persona_system_prompt(self) -> str:
        """Build the system prompt for MICRODRAGON's AI when acting as PA."""
        prompts = {
            PersonaType.DEVELOPER: """You are MICRODRAGON, acting as a senior developer's personal assistant.
You understand: Git workflows, CI/CD pipelines, code review, sprint planning, technical debt.
You communicate: concisely, technically accurately, with code examples when relevant.
You prioritise: shipping working code, unblocking other developers, maintaining code quality.
Always ask: "Will this help ship faster or maintain quality?" before any suggestion.""",

            PersonaType.CEO: """You are MICRODRAGON, acting as a CEO's executive assistant.
You understand: P&L, OKRs, board relationships, investor communications, team dynamics.
You communicate: with executive brevity. One insight = one sentence. No fluff.
You prioritise: decisions with highest business impact, protecting the CEO's time.
You think like: a chief of staff who has seen 100 startups succeed and fail.""",

            PersonaType.TRADER: """You are MICRODRAGON, acting as a trader's personal assistant.
You understand: technical analysis (RSI, MACD, Bollinger Bands), order flow, market structure.
You communicate: with precision. Numbers, percentages, time frames, risk levels.
You always add: "Not financial advice" to market signals.
You think like: a Bloomberg terminal with a brain and a risk manager's discipline.""",

            PersonaType.GAMER: """You are MICRODRAGON, acting as a competitive gamer's assistant.
You understand: meta, tier lists, patch notes, tournament formats, team composition.
You communicate: directly, with gaming terminology. No corporate language.
You prioritise: competitive edge, mechanical improvement, mental game.
You think like: a coach who plays the game at the highest level.""",
        }
        return prompts.get(self.persona, """You are MICRODRAGON personal assistant.
You are helpful, precise, and proactive. You remember everything about the user.
You anticipate their needs and act before being asked when appropriate.""")


# ─── Cross-Platform Memory ────────────────────────────────────────────────────

class CrossPlatformMemory:
    """
    MICRODRAGON remembers conversations across ALL platforms.
    Send a message on WhatsApp → Telegram → Discord → CLI — it's all one context.

    Storage: SQLite with platform tagging
    Format: {platform}_{user_id} → conversation history
    """

    def __init__(self, db_path: str = None):
        self.db_path = db_path or os.path.expanduser(
            "~/.local/share/microdragon/unified_memory.db"
        )
        self._init_db()

    def _init_db(self):
        import sqlite3
        os.makedirs(os.path.dirname(self.db_path), exist_ok=True)
        conn = sqlite3.connect(self.db_path)
        conn.executescript("""
            CREATE TABLE IF NOT EXISTS unified_messages (
                id        INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id   TEXT NOT NULL,
                platform  TEXT NOT NULL,
                role      TEXT NOT NULL,
                content   TEXT NOT NULL,
                timestamp TEXT NOT NULL DEFAULT '',
                session   TEXT DEFAULT ''
            );
            CREATE INDEX IF NOT EXISTS idx_user_platform
                ON unified_messages(user_id, platform, id DESC);
            CREATE INDEX IF NOT EXISTS idx_user_all
                ON unified_messages(user_id, id DESC);

            CREATE TABLE IF NOT EXISTS user_profiles (
                user_id     TEXT PRIMARY KEY,
                name        TEXT DEFAULT '',
                persona     TEXT DEFAULT 'generic',
                preferences TEXT DEFAULT '{}',
                last_seen   TEXT DEFAULT '',
                created_at  TEXT DEFAULT ''
            );

            CREATE TABLE IF NOT EXISTS platform_identities (
                user_id     TEXT NOT NULL,
                platform    TEXT NOT NULL,
                platform_id TEXT NOT NULL,
                PRIMARY KEY (platform, platform_id)
            );
        """)
        conn.commit()
        conn.close()

    def store_message(self, user_id: str, platform: str,
                       role: str, content: str):
        """Store a message from any platform."""
        import sqlite3
        from datetime import datetime
        conn = sqlite3.connect(self.db_path)
        conn.execute(
            """INSERT INTO unified_messages
               (user_id, platform, role, content, timestamp)
               VALUES (?,?,?,?,?)""",
            (user_id, platform, role, content,
             datetime.now().isoformat())
        )
        conn.commit()
        conn.close()

    def get_context(self, user_id: str, limit: int = 20,
                     platform: str = None) -> list[dict]:
        """
        Get conversation context across ALL platforms.
        If platform specified, prioritise that platform's recent messages.
        """
        import sqlite3
        conn = sqlite3.connect(self.db_path)
        if platform:
            rows = conn.execute(
                """SELECT platform, role, content, timestamp
                   FROM unified_messages
                   WHERE user_id = ?
                   ORDER BY id DESC LIMIT ?""",
                (user_id, limit)
            ).fetchall()
        else:
            rows = conn.execute(
                """SELECT platform, role, content, timestamp
                   FROM unified_messages
                   WHERE user_id = ?
                   ORDER BY id DESC LIMIT ?""",
                (user_id, limit)
            ).fetchall()
        conn.close()
        return [
            {
                "platform": r[0],
                "role": r[1],
                "content": r[2],
                "timestamp": r[3]
            }
            for r in reversed(rows)
        ]

    def get_context_string(self, user_id: str, limit: int = 10) -> str:
        """Format context for AI prompt injection."""
        messages = self.get_context(user_id, limit)
        if not messages:
            return ""
        lines = ["[Cross-platform conversation history:]"]
        for msg in messages:
            prefix = f"[{msg['platform'].upper()}]"
            lines.append(f"{prefix} {msg['role'].capitalize()}: {msg['content'][:200]}")
        return '\n'.join(lines)

    def link_platform_identity(self, user_id: str, platform: str,
                                platform_id: str):
        """Link platform-specific IDs to a unified user_id."""
        import sqlite3
        conn = sqlite3.connect(self.db_path)
        conn.execute(
            """INSERT OR REPLACE INTO platform_identities
               (user_id, platform, platform_id) VALUES (?,?,?)""",
            (user_id, platform, platform_id)
        )
        conn.commit()
        conn.close()

    def resolve_user(self, platform: str, platform_id: str) -> str | None:
        """Given a platform-specific ID, find the unified user_id."""
        import sqlite3
        conn = sqlite3.connect(self.db_path)
        row = conn.execute(
            """SELECT user_id FROM platform_identities
               WHERE platform=? AND platform_id=?""",
            (platform, platform_id)
        ).fetchone()
        conn.close()
        return row[0] if row else None

    def set_user_profile(self, user_id: str, name: str = "",
                          persona: str = "generic", preferences: dict = None):
        """Update user profile."""
        import sqlite3, json
        from datetime import datetime
        conn = sqlite3.connect(self.db_path)
        conn.execute(
            """INSERT OR REPLACE INTO user_profiles
               (user_id, name, persona, preferences, last_seen, created_at)
               VALUES (?,?,?,?,?,
                       COALESCE((SELECT created_at FROM user_profiles WHERE user_id=?),
                                ?))""",
            (user_id, name, persona,
             json.dumps(preferences or {}),
             datetime.now().isoformat(),
             user_id, datetime.now().isoformat())
        )
        conn.commit()
        conn.close()

    def get_user_profile(self, user_id: str) -> dict | None:
        """Get user profile."""
        import sqlite3, json
        conn = sqlite3.connect(self.db_path)
        row = conn.execute(
            "SELECT name, persona, preferences, last_seen FROM user_profiles WHERE user_id=?",
            (user_id,)
        ).fetchone()
        conn.close()
        if not row:
            return None
        return {
            "name": row[0],
            "persona": row[1],
            "preferences": json.loads(row[2] or "{}"),
            "last_seen": row[3]
        }

    def clear_history(self, user_id: str, platform: str = None):
        """Clear conversation history for a user."""
        import sqlite3
        conn = sqlite3.connect(self.db_path)
        if platform:
            conn.execute(
                "DELETE FROM unified_messages WHERE user_id=? AND platform=?",
                (user_id, platform)
            )
        else:
            conn.execute(
                "DELETE FROM unified_messages WHERE user_id=?",
                (user_id,)
            )
        conn.commit()
        conn.close()


if __name__ == "__main__":
    import asyncio

    async def demo():
        pa = PersonalAssistantEngine()
        pa.set_persona("developer")
        print(pa.build_persona_system_prompt())
        print("\n" + "─"*50)

        pa.set_persona("trader")
        print(pa.build_briefing_prompt({"portfolio": "AAPL,NVDA,BTC", "pnl": "+3.2%"}))

        print("\n" + "─"*50 + "\nCross-platform memory demo:")
        mem = CrossPlatformMemory("/tmp/microdragon_test.db")
        mem.store_message("user_001", "telegram", "user", "remind me about the investor pitch tomorrow")
        mem.store_message("user_001", "telegram", "assistant", "Noted! I'll remind you about the investor pitch")
        mem.store_message("user_001", "whatsapp", "user", "what was that thing I mentioned on telegram?")

        ctx = mem.get_context_string("user_001")
        print(ctx)

    asyncio.run(demo())
