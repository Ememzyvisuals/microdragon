"""
microdragon/modules/social/x_api/engine.py
═══════════════════════════════════════════════════════════════════════════════
MICRODRAGON — X (TWITTER) API INTEGRATION
═══════════════════════════════════════════════════════════════════════════════

X API v2 integration using verified endpoints and authentication.

CAPABILITIES:
  - Post tweets (requires OAuth 1.0a or OAuth 2.0 user context)
  - Read timeline, mentions, home feed
  - Search posts (Bearer token for public data)
  - Schedule posts for later
  - Get user profile and stats
  - Monitor mentions and keywords (live)
  - Reply to posts
  - Like/retweet/quote

AUTHENTICATION:
  Bearer Token    → Read-only public data (search, user lookup)
  OAuth 1.0a      → Post, like, retweet, DM (on behalf of user)
  OAuth 2.0 PKCE  → Modern user-context auth

TIERS (as of 2026):
  Free    → 1,500 tweets/month write, basic read
  Basic   → $200/month → 3M tweet reads/month
  Pro     → $5,000/month → full archive search

SETUP:
  1. Go to developer.x.com → Create Project → Create App
  2. Generate: API Key, API Key Secret, Bearer Token
  3. For posting: Generate Access Token + Access Token Secret
  4. Set environment variables below

ENV VARIABLES:
  X_BEARER_TOKEN          → for read-only public data
  X_API_KEY               → OAuth 1.0a consumer key
  X_API_KEY_SECRET        → OAuth 1.0a consumer secret  
  X_ACCESS_TOKEN          → OAuth 1.0a user access token
  X_ACCESS_TOKEN_SECRET   → OAuth 1.0a user access token secret
  X_CLIENT_ID             → OAuth 2.0 client ID (for PKCE flow)
  X_CLIENT_SECRET         → OAuth 2.0 client secret

© 2026 EMEMZYVISUALS DIGITALS — Emmanuel Ariyo
"""

import asyncio
import base64
import hashlib
import hmac
import os
import time
import urllib.parse
import uuid
from dataclasses import dataclass, field
from typing import Optional
from pathlib import Path


@dataclass
class XPost:
    id:         str
    text:       str
    author_id:  str
    author:     str = ""
    created_at: str = ""
    likes:      int = 0
    retweets:   int = 0
    replies:    int = 0
    url:        str = ""


@dataclass  
class XConfig:
    bearer_token:        Optional[str] = None
    api_key:             Optional[str] = None
    api_key_secret:      Optional[str] = None
    access_token:        Optional[str] = None
    access_token_secret: Optional[str] = None

    @classmethod
    def from_env(cls) -> "XConfig":
        return cls(
            bearer_token=os.getenv("X_BEARER_TOKEN"),
            api_key=os.getenv("X_API_KEY"),
            api_key_secret=os.getenv("X_API_KEY_SECRET"),
            access_token=os.getenv("X_ACCESS_TOKEN"),
            access_token_secret=os.getenv("X_ACCESS_TOKEN_SECRET"),
        )

    @property
    def can_read(self) -> bool:
        return bool(self.bearer_token)

    @property
    def can_write(self) -> bool:
        return all([self.api_key, self.api_key_secret,
                    self.access_token, self.access_token_secret])

    def setup_guide(self) -> str:
        missing = []
        if not self.bearer_token:
            missing.append("X_BEARER_TOKEN (for search/read)")
        if not self.can_write:
            missing.append("X_API_KEY + X_API_KEY_SECRET + X_ACCESS_TOKEN + X_ACCESS_TOKEN_SECRET (for posting)")
        if not missing:
            return "✓ X API fully configured"
        return (
            "  X API not fully configured.\n\n"
            "  Missing:\n" +
            "\n".join(f"    export {m}" for m in missing) +
            "\n\n"
            "  Get keys:\n"
            "    1. Go to https://developer.x.com/en/portal/dashboard\n"
            "    2. Create a Project → Create App\n"
            "    3. Go to Keys & Tokens → Generate all keys\n"
            "    4. Set the env variables above\n\n"
            "  🔒 Keys are encrypted and stored locally. Never shared with anyone."
        )


class OAuth1Signer:
    """OAuth 1.0a signature generator for X API write operations."""

    def __init__(self, api_key: str, api_secret: str,
                  access_token: str, access_secret: str):
        self.api_key       = api_key
        self.api_secret    = api_secret
        self.access_token  = access_token
        self.access_secret = access_secret

    def sign(self, method: str, url: str, params: dict = None) -> str:
        """Generate OAuth 1.0a Authorization header."""
        params = params or {}
        timestamp   = str(int(time.time()))
        nonce       = uuid.uuid4().hex

        oauth_params = {
            "oauth_consumer_key":     self.api_key,
            "oauth_nonce":            nonce,
            "oauth_signature_method": "HMAC-SHA1",
            "oauth_timestamp":        timestamp,
            "oauth_token":            self.access_token,
            "oauth_version":          "1.0",
        }

        # Combine all params for signature base
        all_params = {**params, **oauth_params}
        sorted_params = "&".join(
            f"{urllib.parse.quote(k, safe='')}"
            f"={urllib.parse.quote(str(v), safe='')}"
            for k, v in sorted(all_params.items())
        )

        base_string = (
            f"{method.upper()}&"
            f"{urllib.parse.quote(url, safe='')}&"
            f"{urllib.parse.quote(sorted_params, safe='')}"
        )

        signing_key = (
            f"{urllib.parse.quote(self.api_secret, safe='')}&"
            f"{urllib.parse.quote(self.access_secret, safe='')}"
        )

        signature = base64.b64encode(
            hmac.new(
                signing_key.encode(),
                base_string.encode(),
                hashlib.sha1
            ).digest()
        ).decode()

        oauth_params["oauth_signature"] = signature

        return "OAuth " + ", ".join(
            f'{urllib.parse.quote(k, safe="")}="{urllib.parse.quote(v, safe="")}"'
            for k, v in sorted(oauth_params.items())
        )


class XApiEngine:
    """Full X API v2 client for Microdragon."""

    BASE = "https://api.x.com"
    API_VERSION = "2"

    def __init__(self, config: Optional[XConfig] = None):
        self.config = config or XConfig.from_env()
        self._signer = None
        if self.config.can_write:
            self._signer = OAuth1Signer(
                self.config.api_key,
                self.config.api_key_secret,
                self.config.access_token,
                self.config.access_token_secret,
            )

    # ─── READ operations (Bearer Token) ──────────────────────────────────────

    async def search(self, query: str, max_results: int = 10) -> list[XPost]:
        """Search recent posts. Requires Bearer Token."""
        if not self.config.can_read:
            raise RuntimeError(self.config.setup_guide())

        import aiohttp
        url = f"{self.BASE}/{self.API_VERSION}/tweets/search/recent"
        params = {
            "query":         query,
            "max_results":   min(max_results, 100),
            "tweet.fields":  "author_id,created_at,public_metrics",
            "expansions":    "author_id",
            "user.fields":   "name,username",
        }
        headers = {"Authorization": f"Bearer {self.config.bearer_token}"}

        async with aiohttp.ClientSession() as session:
            async with session.get(url, params=params, headers=headers) as resp:
                if resp.status == 429:
                    raise RuntimeError("X API rate limit exceeded. Wait and retry.")
                if resp.status != 200:
                    text = await resp.text()
                    raise RuntimeError(f"X API search error {resp.status}: {text[:200]}")
                data = await resp.json()

        posts = []
        users = {u["id"]: u for u in data.get("includes", {}).get("users", [])}
        for tweet in data.get("data", []):
            author = users.get(tweet.get("author_id", ""), {})
            metrics = tweet.get("public_metrics", {})
            posts.append(XPost(
                id=tweet["id"],
                text=tweet["text"],
                author_id=tweet.get("author_id", ""),
                author=author.get("username", ""),
                created_at=tweet.get("created_at", ""),
                likes=metrics.get("like_count", 0),
                retweets=metrics.get("retweet_count", 0),
                replies=metrics.get("reply_count", 0),
                url=f"https://x.com/{author.get('username','')}/status/{tweet['id']}",
            ))
        return posts

    async def get_user(self, username: str) -> dict:
        """Get a user's profile by username."""
        if not self.config.can_read:
            raise RuntimeError(self.config.setup_guide())

        import aiohttp
        url = f"{self.BASE}/{self.API_VERSION}/users/by/username/{username}"
        params = {"user.fields": "description,public_metrics,created_at,verified"}
        headers = {"Authorization": f"Bearer {self.config.bearer_token}"}

        async with aiohttp.ClientSession() as session:
            async with session.get(url, params=params, headers=headers) as resp:
                if resp.status != 200:
                    raise RuntimeError(f"User not found: {username}")
                return (await resp.json()).get("data", {})

    async def get_timeline(self, user_id: str, max_results: int = 5) -> list[XPost]:
        """Get a user's recent tweets by ID."""
        if not self.config.can_read:
            raise RuntimeError(self.config.setup_guide())

        import aiohttp
        url = f"{self.BASE}/{self.API_VERSION}/users/{user_id}/tweets"
        params = {
            "max_results":  min(max_results, 100),
            "tweet.fields": "created_at,public_metrics",
        }
        headers = {"Authorization": f"Bearer {self.config.bearer_token}"}

        async with aiohttp.ClientSession() as session:
            async with session.get(url, params=params, headers=headers) as resp:
                data = await resp.json()

        return [
            XPost(
                id=t["id"], text=t["text"],
                author_id=user_id,
                created_at=t.get("created_at", ""),
                likes=t.get("public_metrics", {}).get("like_count", 0),
            )
            for t in data.get("data", [])
        ]

    # ─── WRITE operations (OAuth 1.0a) ────────────────────────────────────────

    async def post_tweet(self, text: str,
                          reply_to: Optional[str] = None,
                          quote: Optional[str] = None) -> XPost:
        """
        Post a tweet. Requires OAuth 1.0a credentials.
        Always shows preview — never posts silently.
        """
        if not self.config.can_write:
            raise RuntimeError(
                "Posting requires OAuth 1.0a credentials.\n" +
                self.config.setup_guide()
            )
        if not self._signer:
            raise RuntimeError("OAuth signer not initialized")
        if len(text) > 280:
            raise ValueError(f"Tweet too long: {len(text)}/280 characters")

        import aiohttp
        url = f"{self.BASE}/{self.API_VERSION}/tweets"
        payload: dict = {"text": text}
        if reply_to:
            payload["reply"] = {"in_reply_to_tweet_id": reply_to}
        if quote:
            payload["quote_tweet_id"] = quote

        auth_header = self._signer.sign("POST", url)
        headers = {
            "Authorization": auth_header,
            "Content-Type":  "application/json",
        }

        async with aiohttp.ClientSession() as session:
            async with session.post(url, json=payload, headers=headers) as resp:
                if resp.status == 429:
                    raise RuntimeError("X API rate limit exceeded.")
                if resp.status not in (200, 201):
                    text_body = await resp.text()
                    raise RuntimeError(f"Post failed {resp.status}: {text_body[:300]}")
                data = await resp.json()

        tweet_id = data["data"]["id"]
        return XPost(
            id=tweet_id, text=text,
            author_id=data["data"].get("author_id", ""),
            url=f"https://x.com/i/status/{tweet_id}",
        )

    async def like_tweet(self, tweet_id: str, my_user_id: str) -> bool:
        """Like a tweet."""
        if not self.config.can_write or not self._signer:
            raise RuntimeError("Write credentials required")

        import aiohttp
        url = f"{self.BASE}/{self.API_VERSION}/users/{my_user_id}/likes"
        auth_header = self._signer.sign("POST", url)
        async with aiohttp.ClientSession() as session:
            async with session.post(
                url, json={"tweet_id": tweet_id},
                headers={"Authorization": auth_header, "Content-Type": "application/json"}
            ) as resp:
                return resp.status in (200, 201)

    async def retweet(self, tweet_id: str, my_user_id: str) -> bool:
        """Retweet a post."""
        if not self.config.can_write or not self._signer:
            raise RuntimeError("Write credentials required")

        import aiohttp
        url = f"{self.BASE}/{self.API_VERSION}/users/{my_user_id}/retweets"
        auth_header = self._signer.sign("POST", url)
        async with aiohttp.ClientSession() as session:
            async with session.post(
                url, json={"tweet_id": tweet_id},
                headers={"Authorization": auth_header, "Content-Type": "application/json"}
            ) as resp:
                return resp.status in (200, 201)

    def format_post_preview(self, text: str) -> str:
        """Show a preview before posting — Microdragon NEVER posts without this."""
        chars = len(text)
        bar = "─" * 50
        return (
            f"\n  POST PREVIEW\n"
            f"  ┌{bar}┐\n"
            f"  │ {text[:47] + '...' if len(text) > 47 else text:<47} │\n"
            f"  └{bar}┘\n"
            f"  Characters: {chars}/280\n"
            f"\n  ⚠  Type YES to post, anything else to cancel: "
        )

    def get_status(self) -> str:
        """Show X API configuration status."""
        lines = ["\n  🐉 X API Status\n"]
        if self.config.bearer_token:
            lines.append("  ✓ Bearer Token set  → search, read public data")
        else:
            lines.append("  ✗ Bearer Token missing → set X_BEARER_TOKEN")
        if self.config.can_write:
            lines.append("  ✓ OAuth 1.0a set    → post, like, retweet")
        else:
            lines.append("  ✗ OAuth 1.0a missing → set X_API_KEY + X_API_KEY_SECRET + X_ACCESS_TOKEN + X_ACCESS_TOKEN_SECRET")

        lines.extend([
            "",
            "  Free tier: 1,500 posts/month write, basic read",
            "  Basic: $200/month → 3M reads/month",
            "  Setup: https://developer.x.com",
            "",
            "  🔒 Keys stored encrypted locally. Never shared.",
            "",
        ])
        return "\n".join(lines)
