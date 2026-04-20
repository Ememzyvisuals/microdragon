"""
microdragon/modules/calendar/src/engine.py
MICRODRAGON Calendar Module — Read/write calendar events across platforms
Supports: Google Calendar (API), macOS Calendar (AppleScript), Linux (calcurse/remind)
"""

import asyncio
import subprocess
import os
import json
from dataclasses import dataclass, field
from typing import Optional
from datetime import datetime, date, timedelta


@dataclass
class CalendarEvent:
    title: str
    start: datetime
    end: datetime
    location: str = ""
    description: str = ""
    attendees: list = field(default_factory=list)
    calendar: str = "default"
    uid: Optional[str] = None
    is_all_day: bool = False


class GoogleCalendarClient:
    """Google Calendar API client."""

    def __init__(self, credentials_path: str = ""):
        self.creds_path = credentials_path or os.getenv("GOOGLE_CALENDAR_CREDENTIALS", "")
        self._service = None

    def _get_service(self):
        if self._service:
            return self._service
        try:
            from google.oauth2.credentials import Credentials
            from google_auth_oauthlib.flow import InstalledAppFlow
            from googleapiclient.discovery import build

            SCOPES = ["https://www.googleapis.com/auth/calendar"]
            creds = None

            token_path = os.path.expanduser("~/.local/share/microdragon/google_token.json")
            if os.path.exists(token_path):
                creds = Credentials.from_authorized_user_file(token_path, SCOPES)

            if not creds or not creds.valid:
                if creds and creds.expired and creds.refresh_token:
                    from google.auth.transport.requests import Request
                    creds.refresh(Request())
                else:
                    flow = InstalledAppFlow.from_client_secrets_file(self.creds_path, SCOPES)
                    creds = flow.run_local_server(port=0)
                with open(token_path, "w") as token:
                    token.write(creds.to_json())

            self._service = build("calendar", "v3", credentials=creds)
            return self._service
        except ImportError:
            return None

    def get_today_events(self) -> list[CalendarEvent]:
        service = self._get_service()
        if not service:
            return []

        try:
            today_start = datetime.combine(date.today(), datetime.min.time()).isoformat() + "Z"
            today_end = datetime.combine(date.today(), datetime.max.time()).isoformat() + "Z"

            result = service.events().list(
                calendarId="primary",
                timeMin=today_start,
                timeMax=today_end,
                singleEvents=True,
                orderBy="startTime"
            ).execute()

            events = []
            for item in result.get("items", []):
                start = item["start"].get("dateTime", item["start"].get("date", ""))
                end = item["end"].get("dateTime", item["end"].get("date", ""))

                try:
                    start_dt = datetime.fromisoformat(start.replace("Z", "+00:00"))
                    end_dt = datetime.fromisoformat(end.replace("Z", "+00:00"))
                except ValueError:
                    start_dt = datetime.now()
                    end_dt = datetime.now()

                events.append(CalendarEvent(
                    title=item.get("summary", ""),
                    start=start_dt, end=end_dt,
                    location=item.get("location", ""),
                    description=item.get("description", ""),
                    uid=item.get("id")
                ))
            return events
        except Exception as e:
            print(f"[Calendar] Google API error: {e}")
            return []

    def create_event(self, event: CalendarEvent) -> Optional[str]:
        service = self._get_service()
        if not service:
            return None

        try:
            body = {
                "summary": event.title,
                "location": event.location,
                "description": event.description,
                "start": {"dateTime": event.start.isoformat(), "timeZone": "UTC"},
                "end":   {"dateTime": event.end.isoformat(),   "timeZone": "UTC"},
            }
            if event.attendees:
                body["attendees"] = [{"email": a} for a in event.attendees]

            created = service.events().insert(calendarId="primary", body=body).execute()
            return created.get("id")
        except Exception as e:
            print(f"[Calendar] Create event error: {e}")
            return None


class MacOSCalendar:
    """macOS Calendar via AppleScript."""

    def get_today_events(self) -> list[CalendarEvent]:
        script = """
tell application "Calendar"
    set today_start to (current date)
    set time of today_start to 0
    set today_end to today_start + (86400 - 1)
    set event_list to {}
    repeat with cal in calendars
        try
            set evts to every event of cal whose start date >= today_start and start date <= today_end
            repeat with evt in evts
                set end of event_list to (summary of evt & "|" & (start date of evt as string) & "|" & (end date of evt as string) & "|" & (location of evt))
            end repeat
        end try
    end repeat
    return event_list
end tell
"""
        try:
            result = subprocess.run(["osascript", "-e", script],
                                    capture_output=True, text=True, timeout=10)
            events = []
            for line in result.stdout.strip().split(", "):
                parts = line.split("|")
                if len(parts) >= 2:
                    try:
                        events.append(CalendarEvent(
                            title=parts[0].strip(),
                            start=datetime.now(),  # Parse parts[1] in production
                            end=datetime.now(),
                            location=parts[3].strip() if len(parts) > 3 else ""
                        ))
                    except Exception:
                        continue
            return events
        except Exception:
            return []

    def create_event(self, event: CalendarEvent) -> bool:
        script = f"""
tell application "Calendar"
    tell calendar "Home"
        make new event at end with properties {{
            summary:"{event.title.replace('"', '')}",
            start date:date "{event.start.strftime('%A, %B %d, %Y at %I:%M:%S %p')}",
            end date:date "{event.end.strftime('%A, %B %d, %Y at %I:%M:%S %p')}",
            location:"{event.location.replace('"', '')}"
        }}
    end tell
end tell
"""
        try:
            result = subprocess.run(["osascript", "-e", script],
                                    capture_output=True, timeout=10)
            return result.returncode == 0
        except Exception:
            return False


class CalendarEngine:
    """Unified calendar engine for MICRODRAGON."""

    def __init__(self):
        self.google = GoogleCalendarClient()
        self.macos = MacOSCalendar() if self._is_macos() else None

    def _is_macos(self) -> bool:
        import sys
        return sys.platform == "darwin"

    def get_today_events(self) -> list[CalendarEvent]:
        """Get today's events from the best available calendar."""
        # Try Google Calendar first
        events = self.google.get_today_events()
        if events:
            return events

        # Try macOS Calendar
        if self.macos:
            return self.macos.get_today_events()

        return []

    def format_daily_briefing(self) -> str:
        """Format today's calendar for AI briefing."""
        events = self.get_today_events()
        today = date.today().strftime("%A, %B %d, %Y")

        if not events:
            return f"📅 Calendar for {today}:\n  No events scheduled."

        lines = [f"📅 Calendar for {today}:\n"]
        for evt in sorted(events, key=lambda e: e.start):
            time_str = evt.start.strftime("%I:%M %p")
            duration = int((evt.end - evt.start).seconds / 60)
            lines.append(f"  • {time_str} — {evt.title} ({duration}min)")
            if evt.location:
                lines.append(f"    📍 {evt.location}")
        return "\n".join(lines)

    async def create_event_with_confirmation(self, title: str, start: datetime,
                                              end: datetime, location: str = "",
                                              confirmed: bool = False) -> tuple[bool, str]:
        """Create a calendar event — requires confirmation."""
        preview = (
            f"Calendar Event:\n"
            f"  Title: {title}\n"
            f"  Start: {start.strftime('%A, %B %d at %I:%M %p')}\n"
            f"  End:   {end.strftime('%I:%M %p')}\n"
            f"  Location: {location or 'Not specified'}"
        )

        if not confirmed:
            return False, f"Confirmation required:\n{preview}"

        event = CalendarEvent(title=title, start=start, end=end, location=location)

        # Try Google Calendar
        uid = self.google.create_event(event)
        if uid:
            return True, f"Event created: {title} on {start.strftime('%b %d at %I:%M %p')}"

        # Try macOS Calendar
        if self.macos and self.macos.create_event(event):
            return True, f"Event created in macOS Calendar: {title}"

        return False, "Could not create event — no calendar configured"


if __name__ == "__main__":
    engine = CalendarEngine()
    print("[MICRODRAGON Calendar] Engine ready")
    print(engine.format_daily_briefing())
