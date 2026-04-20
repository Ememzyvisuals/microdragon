"""
microdragon/modules/github/src/engine.py
MICRODRAGON GitHub Module — PR review, issue management, repo analysis, CI monitoring
"""

import asyncio
import aiohttp
import os
import subprocess
import json
from dataclasses import dataclass, field
from typing import Optional


@dataclass
class PullRequest:
    number: int
    title: str
    description: str
    author: str
    diff: str
    files_changed: list
    additions: int
    deletions: int
    base_branch: str
    head_branch: str
    url: str


@dataclass
class ReviewComment:
    file: str
    line: int
    severity: str      # CRITICAL | HIGH | MEDIUM | LOW | INFO
    category: str      # security | performance | style | logic | test
    message: str
    suggestion: Optional[str] = None


@dataclass
class CodeReview:
    pr: PullRequest
    overall_verdict: str      # APPROVE | REQUEST_CHANGES | COMMENT
    summary: str
    comments: list[ReviewComment] = field(default_factory=list)
    security_issues: int = 0
    logic_issues: int = 0
    style_issues: int = 0
    score: int = 0


class GitHubClient:
    """GitHub REST API client."""

    BASE = "https://api.github.com"

    def __init__(self, token: str):
        self.token = token
        self.headers = {
            "Authorization": f"token {token}",
            "Accept": "application/vnd.github.v3+json",
            "X-GitHub-Api-Version": "2022-11-28"
        }

    async def get_pr(self, owner: str, repo: str, pr_number: int) -> Optional[PullRequest]:
        async with aiohttp.ClientSession() as session:
            # Get PR metadata
            async with session.get(
                f"{self.BASE}/repos/{owner}/{repo}/pulls/{pr_number}",
                headers=self.headers
            ) as resp:
                if resp.status != 200:
                    return None
                pr_data = await resp.json()

            # Get diff
            diff_headers = {**self.headers, "Accept": "application/vnd.github.v3.diff"}
            async with session.get(
                f"{self.BASE}/repos/{owner}/{repo}/pulls/{pr_number}",
                headers=diff_headers
            ) as resp:
                diff = await resp.text() if resp.status == 200 else ""

            # Get changed files
            async with session.get(
                f"{self.BASE}/repos/{owner}/{repo}/pulls/{pr_number}/files",
                headers=self.headers
            ) as resp:
                files_data = await resp.json() if resp.status == 200 else []

            return PullRequest(
                number=pr_number,
                title=pr_data.get("title", ""),
                description=pr_data.get("body", "") or "",
                author=pr_data.get("user", {}).get("login", ""),
                diff=diff[:50000],  # Cap diff size
                files_changed=[f["filename"] for f in files_data],
                additions=pr_data.get("additions", 0),
                deletions=pr_data.get("deletions", 0),
                base_branch=pr_data.get("base", {}).get("ref", ""),
                head_branch=pr_data.get("head", {}).get("ref", ""),
                url=pr_data.get("html_url", "")
            )

    async def post_review(self, owner: str, repo: str, pr_number: int,
                           review: CodeReview) -> bool:
        """Post AI review to GitHub PR."""
        # Build review body
        body = f"## MICRODRAGON AI Code Review\n\n"
        body += f"**Verdict:** {review.overall_verdict}\n\n"
        body += f"### Summary\n{review.summary}\n\n"

        if review.comments:
            body += f"### Issues Found\n"
            for c in review.comments[:10]:
                emoji = {"CRITICAL": "🚨", "HIGH": "⚠️", "MEDIUM": "⚡", "LOW": "ℹ️"}.get(c.severity, "•")
                body += f"\n**{emoji} {c.severity} — {c.category}** (`{c.file}:{c.line}`)\n"
                body += f"{c.message}\n"
                if c.suggestion:
                    body += f"\n*Suggestion:* {c.suggestion}\n"

        body += f"\n---\n*Review by MICRODRAGON AI Agent — not a substitute for human review*"

        event = "APPROVE" if review.overall_verdict == "APPROVE" else "COMMENT"

        async with aiohttp.ClientSession() as session:
            async with session.post(
                f"{self.BASE}/repos/{owner}/{repo}/pulls/{pr_number}/reviews",
                headers=self.headers,
                json={"body": body, "event": event}
            ) as resp:
                return resp.status in (200, 201)

    async def create_issue(self, owner: str, repo: str, title: str,
                            body: str, labels: list = None) -> Optional[str]:
        """Create a GitHub issue."""
        async with aiohttp.ClientSession() as session:
            async with session.post(
                f"{self.BASE}/repos/{owner}/{repo}/issues",
                headers=self.headers,
                json={"title": title, "body": body, "labels": labels or []}
            ) as resp:
                if resp.status == 201:
                    data = await resp.json()
                    return data.get("html_url")
                return None

    async def get_repo_stats(self, owner: str, repo: str) -> dict:
        """Get repository statistics."""
        async with aiohttp.ClientSession() as session:
            async with session.get(
                f"{self.BASE}/repos/{owner}/{repo}",
                headers=self.headers
            ) as resp:
                if resp.status != 200:
                    return {}
                data = await resp.json()
                return {
                    "name": data.get("name"),
                    "description": data.get("description"),
                    "stars": data.get("stargazers_count", 0),
                    "forks": data.get("forks_count", 0),
                    "open_issues": data.get("open_issues_count", 0),
                    "language": data.get("language"),
                    "size_kb": data.get("size", 0),
                    "default_branch": data.get("default_branch"),
                    "url": data.get("html_url"),
                }

    async def list_open_prs(self, owner: str, repo: str) -> list[dict]:
        """List open pull requests."""
        async with aiohttp.ClientSession() as session:
            async with session.get(
                f"{self.BASE}/repos/{owner}/{repo}/pulls?state=open&per_page=20",
                headers=self.headers
            ) as resp:
                if resp.status != 200:
                    return []
                prs = await resp.json()
                return [{
                    "number": pr["number"],
                    "title": pr["title"],
                    "author": pr["user"]["login"],
                    "url": pr["html_url"],
                    "created": pr["created_at"][:10],
                } for pr in prs]


class PRReviewer:
    """AI-powered pull request reviewer."""

    # Patterns that indicate security issues
    SECURITY_PATTERNS = [
        ("sql injection", "eval(", "exec(", "os.system("),
        ("hardcoded password", "password =", "secret =", "api_key ="),
        ("xss", "innerHTML", "document.write(", "dangerouslySetInnerHTML"),
        ("insecure", "verify=False", "ssl_verify=False", "check_certificate"),
        ("command injection", "subprocess.call(shell=True", "os.popen("),
        ("path traversal", "../", "..\\\\"),
        ("debug", "DEBUG=True", "debug: true"),
    ]

    def analyze_diff(self, diff: str) -> list[ReviewComment]:
        """Static analysis of a diff for common issues."""
        comments = []
        lines = diff.split("\n")

        for i, line in enumerate(lines):
            if not line.startswith("+") or line.startswith("+++"):
                continue

            code_line = line[1:].strip().lower()
            line_num = i

            # Security checks
            for patterns in self.SECURITY_PATTERNS:
                for pattern in patterns:
                    if pattern in code_line:
                        comments.append(ReviewComment(
                            file="diff", line=line_num,
                            severity="HIGH", category="security",
                            message=f"Potential security issue: '{pattern}' detected",
                            suggestion="Review this line for security implications"
                        ))
                        break

            # TODO/FIXME
            if "todo" in code_line or "fixme" in code_line or "hack" in code_line:
                comments.append(ReviewComment(
                    file="diff", line=line_num,
                    severity="LOW", category="style",
                    message="TODO/FIXME/HACK comment in production code",
                    suggestion="Resolve or create a ticket for this"
                ))

            # Long lines
            if len(line) > 150:
                comments.append(ReviewComment(
                    file="diff", line=line_num,
                    severity="LOW", category="style",
                    message=f"Line too long ({len(line)} chars, max 120 recommended)",
                    suggestion="Break into multiple lines for readability"
                ))

        return comments[:20]  # Cap at 20 static comments

    def build_review_prompt(self, pr: PullRequest) -> str:
        """Build a prompt for AI to review the PR."""
        return f"""You are a senior software engineer performing a code review.

**Pull Request #{pr.number}: {pr.title}**
Author: {pr.author}
Branch: {pr.head_branch} → {pr.base_branch}
Files changed: {', '.join(pr.files_changed[:10])}
+{pr.additions} / -{pr.deletions}

**Description:**
{pr.description[:500]}

**Diff (key sections):**
```
{pr.diff[:8000]}
```

Provide a thorough code review covering:
1. **Security vulnerabilities** (SQL injection, XSS, secrets, SSRF, etc.)
2. **Logic errors** (edge cases, null handling, race conditions)
3. **Performance issues** (N+1 queries, unnecessary loops, memory leaks)
4. **Test coverage** (missing tests, edge cases not covered)
5. **Code quality** (readability, naming, complexity)
6. **Breaking changes** (API compatibility, schema changes)

Format your review as:
- Overall verdict: APPROVE | REQUEST_CHANGES | COMMENT
- Summary (2-3 sentences)
- Specific issues with file:line references where possible
- Actionable suggestions for each issue

Be thorough but constructive. This is production code."""


class GitHubEngine:
    """Unified GitHub engine for MICRODRAGON."""

    def __init__(self):
        self.token = os.getenv("GITHUB_TOKEN", "")
        self.client = GitHubClient(self.token) if self.token else None
        self.reviewer = PRReviewer()

    def is_configured(self) -> bool:
        return bool(self.token)

    def parse_pr_url(self, url: str) -> tuple[str, str, int]:
        """Parse a GitHub PR URL into owner, repo, number."""
        # https://github.com/owner/repo/pull/123
        parts = url.replace("https://github.com/", "").split("/")
        if len(parts) >= 4 and parts[2] == "pull":
            return parts[0], parts[1], int(parts[3])
        raise ValueError(f"Invalid PR URL: {url}")

    async def review_pr(self, pr_url: str, post_review: bool = False) -> str:
        """Full AI review of a pull request."""
        if not self.client:
            return "GitHub token not configured. Set GITHUB_TOKEN."

        try:
            owner, repo, number = self.parse_pr_url(pr_url)
            pr = await self.client.get_pr(owner, repo, number)
            if not pr:
                return f"PR #{number} not found or not accessible."

            # Static analysis
            static_comments = self.reviewer.analyze_diff(pr.diff)

            # Build AI review prompt
            prompt = self.reviewer.build_review_prompt(pr)

            return (
                f"## PR Review: #{pr.number} — {pr.title}\n\n"
                f"**Author:** {pr.author}\n"
                f"**Changes:** +{pr.additions}/-{pr.deletions} in {len(pr.files_changed)} files\n\n"
                f"**Static Analysis Found:** {len(static_comments)} issues\n"
                + "\n".join(f"- [{c.severity}] {c.message}" for c in static_comments[:5])
                + f"\n\n---\n{prompt[:3000]}"
            )
        except Exception as e:
            return f"Review error: {e}"

    async def create_issue(self, owner: str, repo: str, title: str,
                            description: str) -> str:
        if not self.client:
            return "GitHub token not configured."
        url = await self.client.create_issue(owner, repo, title, description)
        return f"Issue created: {url}" if url else "Failed to create issue"

    async def repo_overview(self, owner: str, repo: str) -> str:
        if not self.client:
            return "GitHub token not configured."
        stats = await self.client.get_repo_stats(owner, repo)
        prs = await self.client.list_open_prs(owner, repo)

        return (
            f"## Repository: {owner}/{repo}\n\n"
            f"⭐ Stars: {stats.get('stars', 0):,}\n"
            f"🔀 Forks: {stats.get('forks', 0):,}\n"
            f"🐛 Open Issues: {stats.get('open_issues', 0):,}\n"
            f"💻 Language: {stats.get('language', 'Unknown')}\n"
            f"📦 Size: {stats.get('size_kb', 0):,} KB\n\n"
            f"**Open PRs ({len(prs)}):**\n"
            + "\n".join(f"  #{pr['number']}: {pr['title']} (@{pr['author']})" for pr in prs[:5])
        )


if __name__ == "__main__":
    async def demo():
        engine = GitHubEngine()
        print(f"[MICRODRAGON GitHub] Token configured: {engine.is_configured()}")
        if not engine.is_configured():
            print("  Set GITHUB_TOKEN environment variable")

    asyncio.run(demo())
