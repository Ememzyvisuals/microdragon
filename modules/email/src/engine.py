"""
microdragon/modules/email/src/engine.py
MICRODRAGON Email Module — IMAP reading + SMTP sending with mandatory confirmation
UNLIKE OpenClaw: every send requires explicit user confirmation (no silent emails)
"""

import asyncio
import imaplib
import smtplib
import email
from email.mime.text import MIMEText
from email.mime.multipart import MIMEMultipart
from email.mime.base import MIMEBase
from email import encoders
from email.header import decode_header
import os
from dataclasses import dataclass, field
from typing import Optional
from datetime import datetime


@dataclass
class EmailMessage:
    uid: str
    subject: str
    sender: str
    recipients: list
    body: str
    date: str
    is_read: bool = False
    attachments: list = field(default_factory=list)
    thread_id: Optional[str] = None


@dataclass
class EmailConfig:
    imap_server: str
    imap_port: int
    smtp_server: str
    smtp_port: int
    email: str
    password: str
    use_ssl: bool = True


class EmailReader:
    """IMAP email reader with smart summarization."""

    def __init__(self, config: EmailConfig):
        self.config = config

    def _connect_imap(self) -> imaplib.IMAP4_SSL:
        if self.config.use_ssl:
            conn = imaplib.IMAP4_SSL(self.config.imap_server, self.config.imap_port)
        else:
            conn = imaplib.IMAP4(self.config.imap_server, self.config.imap_port)
        conn.login(self.config.email, self.config.password)
        return conn

    def get_unread(self, folder: str = "INBOX", limit: int = 20) -> list[EmailMessage]:
        """Fetch unread emails."""
        messages = []
        try:
            conn = self._connect_imap()
            conn.select(folder)

            _, uids = conn.search(None, "UNSEEN")
            uid_list = uids[0].split()[-limit:]  # Most recent N unread

            for uid in reversed(uid_list):
                try:
                    _, data = conn.fetch(uid, "(RFC822)")
                    if not data or not data[0]: continue

                    raw = data[0][1]
                    msg = email.message_from_bytes(raw)

                    subject = self._decode_header(msg.get("Subject", ""))
                    sender = msg.get("From", "")
                    date = msg.get("Date", "")
                    body = self._extract_body(msg)

                    messages.append(EmailMessage(
                        uid=uid.decode(),
                        subject=subject,
                        sender=sender,
                        recipients=[msg.get("To", "")],
                        body=body[:2000],
                        date=date,
                        is_read=False
                    ))
                except Exception:
                    continue

            conn.logout()
        except Exception as e:
            print(f"[Email] IMAP error: {e}")

        return messages

    def search(self, query: str, folder: str = "INBOX", limit: int = 10) -> list[EmailMessage]:
        """Search emails by subject or body."""
        messages = []
        try:
            conn = self._connect_imap()
            conn.select(folder)

            # IMAP search
            search_query = f'(SUBJECT "{query}")'
            _, uids = conn.search(None, search_query)
            uid_list = uids[0].split()[-limit:]

            for uid in reversed(uid_list):
                try:
                    _, data = conn.fetch(uid, "(RFC822)")
                    raw = data[0][1]
                    msg = email.message_from_bytes(raw)
                    body = self._extract_body(msg)
                    messages.append(EmailMessage(
                        uid=uid.decode(),
                        subject=self._decode_header(msg.get("Subject", "")),
                        sender=msg.get("From", ""),
                        recipients=[msg.get("To", "")],
                        body=body[:2000],
                        date=msg.get("Date", ""),
                    ))
                except Exception:
                    continue
            conn.logout()
        except Exception as e:
            print(f"[Email] Search error: {e}")

        return messages

    def _decode_header(self, raw: str) -> str:
        decoded, encoding = decode_header(raw)[0]
        if isinstance(decoded, bytes):
            return decoded.decode(encoding or "utf-8", errors="replace")
        return str(decoded)

    def _extract_body(self, msg) -> str:
        """Extract plain text body from email."""
        if msg.is_multipart():
            for part in msg.walk():
                ct = part.get_content_type()
                if ct == "text/plain":
                    try:
                        return part.get_payload(decode=True).decode(
                            part.get_content_charset() or "utf-8", errors="replace"
                        )
                    except Exception:
                        pass
        else:
            try:
                return msg.get_payload(decode=True).decode(
                    msg.get_content_charset() or "utf-8", errors="replace"
                )
            except Exception:
                pass
        return ""

    def summarize_inbox(self, messages: list[EmailMessage]) -> str:
        """Create a human-readable inbox summary."""
        if not messages:
            return "Inbox is empty."

        lines = [f"📬 {len(messages)} unread emails:\n"]
        for i, msg in enumerate(messages[:10], 1):
            sender_name = msg.sender.split("<")[0].strip().strip('"') or msg.sender
            subject_short = msg.subject[:60]
            lines.append(f"  {i}. From: {sender_name}")
            lines.append(f"     Subject: {subject_short}")
            lines.append(f"     Date: {msg.date[:20] if msg.date else 'Unknown'}")
            if msg.body:
                preview = msg.body[:100].replace("\n", " ").strip()
                lines.append(f"     Preview: {preview}...")
            lines.append("")

        return "\n".join(lines)


class EmailSender:
    """
    SMTP email sender with MANDATORY confirmation gate.
    MICRODRAGON NEVER sends email without explicit user approval.
    This directly prevents the OpenClaw "deleted Gmail inbox" class of incidents.
    """

    def __init__(self, config: EmailConfig):
        self.config = config
        self._pending_queue: list[dict] = []

    def draft(self, to: str, subject: str, body: str,
               cc: Optional[str] = None, attachments: list = None) -> dict:
        """Create a draft — does NOT send yet."""
        draft = {
            "id": str(int(datetime.now().timestamp())),
            "to": to, "subject": subject, "body": body,
            "cc": cc or "", "attachments": attachments or [],
            "status": "draft",
            "created_at": datetime.now().isoformat()
        }
        self._pending_queue.append(draft)
        return draft

    def preview(self, draft: dict) -> str:
        """Show a clear preview of the email before sending."""
        separator = "─" * 50
        return f"""
╔{separator}╗
║  EMAIL DRAFT — REQUIRES YOUR CONFIRMATION          ║
╠{separator}╣
║  To:      {draft['to'][:44]:<44} ║
║  CC:      {(draft.get('cc', '') or 'none')[:44]:<44} ║
║  Subject: {draft['subject'][:44]:<44} ║
╠{separator}╣
  {chr(10).join('  ' + line for line in draft['body'][:500].split(chr(10)))}
╠{separator}╣
║  Attachments: {len(draft.get('attachments', []))} file(s)     ║
╚{separator}╝

⚠  Type 'yes' to send, anything else to cancel.
"""

    def send_confirmed(self, draft: dict, user_confirmed: bool = False) -> bool:
        """
        Send the email — ONLY if user_confirmed=True.
        This is the core safety gate. Never bypass this.
        """
        if not user_confirmed:
            print("[Email] Send cancelled — explicit confirmation required.")
            return False

        try:
            msg = MIMEMultipart()
            msg["From"] = self.config.email
            msg["To"] = draft["to"]
            msg["Subject"] = draft["subject"]
            if draft.get("cc"):
                msg["Cc"] = draft["cc"]

            msg.attach(MIMEText(draft["body"], "plain", "utf-8"))

            # Attach files
            for attachment_path in draft.get("attachments", []):
                if os.path.exists(attachment_path):
                    with open(attachment_path, "rb") as f:
                        part = MIMEBase("application", "octet-stream")
                        part.set_payload(f.read())
                    encoders.encode_base64(part)
                    part.add_header("Content-Disposition",
                                    f"attachment; filename={os.path.basename(attachment_path)}")
                    msg.attach(part)

            # Connect and send
            if self.config.use_ssl and self.config.smtp_port == 465:
                with smtplib.SMTP_SSL(self.config.smtp_server, self.config.smtp_port) as smtp:
                    smtp.login(self.config.email, self.config.password)
                    smtp.send_message(msg)
            else:
                with smtplib.SMTP(self.config.smtp_server, self.config.smtp_port) as smtp:
                    smtp.starttls()
                    smtp.login(self.config.email, self.config.password)
                    smtp.send_message(msg)

            draft["status"] = "sent"
            print(f"[Email] Sent to {draft['to']}: {draft['subject']}")
            return True

        except Exception as e:
            print(f"[Email] Send failed: {e}")
            return False


class EmailEngine:
    """Unified email engine — compose, read, reply, organize."""

    def __init__(self):
        self.config = self._load_config()
        if self.config:
            self.reader = EmailReader(self.config)
            self.sender = EmailSender(self.config)
        else:
            self.reader = None
            self.sender = None

    def _load_config(self) -> Optional[EmailConfig]:
        # Load from environment or config file
        email_addr = os.getenv("MICRODRAGON_EMAIL")
        password = os.getenv("MICRODRAGON_EMAIL_PASSWORD")
        if not email_addr or not password:
            return None

        # Auto-detect settings for common providers
        domain = email_addr.split("@")[-1].lower()
        presets = {
            "gmail.com":     ("imap.gmail.com", 993, "smtp.gmail.com", 587),
            "outlook.com":   ("outlook.office365.com", 993, "smtp.office365.com", 587),
            "hotmail.com":   ("outlook.office365.com", 993, "smtp.office365.com", 587),
            "yahoo.com":     ("imap.mail.yahoo.com", 993, "smtp.mail.yahoo.com", 587),
            "protonmail.com":("127.0.0.1", 1143, "127.0.0.1", 1025),  # Bridge
        }

        imap_h, imap_p, smtp_h, smtp_p = presets.get(domain, ("imap."+domain, 993, "smtp."+domain, 587))
        return EmailConfig(imap_h, imap_p, smtp_h, smtp_p, email_addr, password)

    def is_configured(self) -> bool:
        return self.config is not None

    def get_inbox_summary(self, limit: int = 20) -> str:
        if not self.reader:
            return "Email not configured. Set MICRODRAGON_EMAIL and MICRODRAGON_EMAIL_PASSWORD."
        messages = self.reader.get_unread(limit=limit)
        return self.reader.summarize_inbox(messages)

    def compose_draft(self, to: str, subject: str, body: str,
                       cc: str = "", attachments: list = None) -> dict:
        if not self.sender:
            return {"error": "Email not configured"}
        return self.sender.draft(to, subject, body, cc, attachments)

    def preview_draft(self, draft: dict) -> str:
        if not self.sender:
            return "Email not configured"
        return self.sender.preview(draft)

    def send_with_confirmation(self, draft: dict, confirmed: bool) -> bool:
        if not self.sender:
            return False
        return self.sender.send_confirmed(draft, user_confirmed=confirmed)


if __name__ == "__main__":
    engine = EmailEngine()
    print(f"[MICRODRAGON Email] Configured: {engine.is_configured()}")
    if not engine.is_configured():
        print("  Set: MICRODRAGON_EMAIL=you@gmail.com MICRODRAGON_EMAIL_PASSWORD=your_app_password")
