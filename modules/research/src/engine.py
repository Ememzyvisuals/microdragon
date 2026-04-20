"""
microdragon/modules/research/src/engine.py
MICRODRAGON Research Module — Web crawling, extraction, summarization
"""

import asyncio
import aiohttp
import json
import re
from dataclasses import dataclass, field
from typing import Optional
from urllib.parse import urlparse, urljoin, quote_plus
from html.parser import HTMLParser


@dataclass
class ResearchResult:
    query: str
    sources: list = field(default_factory=list)
    summary: str = ""
    key_facts: list = field(default_factory=list)
    confidence: float = 0.0


@dataclass
class WebSource:
    url: str
    title: str
    content: str
    domain: str
    word_count: int = 0
    relevance_score: float = 0.0


class HTMLTextExtractor(HTMLParser):
    """Fast HTML → plain text extractor without heavy deps."""

    SKIP_TAGS = {"script", "style", "nav", "footer", "header", "aside", "noscript"}

    def __init__(self):
        super().__init__()
        self.text_parts = []
        self._skip = False
        self._skip_depth = 0
        self._current_tag = ""

    def handle_starttag(self, tag, attrs):
        self._current_tag = tag
        if tag in self.SKIP_TAGS:
            self._skip = True
            self._skip_depth += 1

    def handle_endtag(self, tag):
        if tag in self.SKIP_TAGS:
            self._skip_depth -= 1
            if self._skip_depth <= 0:
                self._skip = False
                self._skip_depth = 0

    def handle_data(self, data):
        if not self._skip:
            stripped = data.strip()
            if len(stripped) > 10:
                self.text_parts.append(stripped)

    def get_text(self) -> str:
        return "\n".join(self.text_parts)


class WebCrawler:
    """Async web crawler for MICRODRAGON research module."""

    HEADERS = {
        "User-Agent": "Mozilla/5.0 (compatible; MICRODRAGONResearch/1.0)",
        "Accept": "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
        "Accept-Language": "en-US,en;q=0.5",
    }

    def __init__(self, timeout: int = 15, max_content_kb: int = 200):
        self.timeout = aiohttp.ClientTimeout(total=timeout)
        self.max_bytes = max_content_kb * 1024

    async def fetch(self, url: str) -> Optional[WebSource]:
        try:
            async with aiohttp.ClientSession(headers=self.HEADERS, timeout=self.timeout) as session:
                async with session.get(url) as resp:
                    if resp.status != 200:
                        return None

                    content_type = resp.headers.get("Content-Type", "")
                    if "text/html" not in content_type and "text/plain" not in content_type:
                        return None

                    raw = await resp.read()
                    # Limit size
                    raw = raw[:self.max_bytes]
                    html = raw.decode("utf-8", errors="replace")

                    # Extract title
                    title_match = re.search(r"<title[^>]*>(.*?)</title>", html, re.IGNORECASE | re.DOTALL)
                    title = title_match.group(1).strip() if title_match else urlparse(url).netloc

                    # Extract text
                    extractor = HTMLTextExtractor()
                    extractor.feed(html)
                    text = extractor.get_text()

                    domain = urlparse(url).netloc
                    return WebSource(
                        url=url, title=title, content=text,
                        domain=domain, word_count=len(text.split())
                    )
        except Exception:
            return None

    async def fetch_many(self, urls: list[str], max_concurrent: int = 5) -> list[WebSource]:
        semaphore = asyncio.Semaphore(max_concurrent)

        async def fetch_with_sem(url):
            async with semaphore:
                return await self.fetch(url)

        results = await asyncio.gather(*[fetch_with_sem(url) for url in urls])
        return [r for r in results if r is not None]


class SearchEngine:
    """Multi-engine web search wrapper (DuckDuckGo, Brave, SearXNG)."""

    DDGO_URL = "https://html.duckduckgo.com/html/"

    def __init__(self):
        self.timeout = aiohttp.ClientTimeout(total=10)

    async def search_ddg(self, query: str, max_results: int = 8) -> list[str]:
        """DuckDuckGo HTML search — no API key needed."""
        urls = []
        try:
            payload = {"q": query, "b": "", "kl": "en-us"}
            headers = {
                "User-Agent": "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36",
                "Content-Type": "application/x-www-form-urlencoded",
            }
            async with aiohttp.ClientSession(timeout=self.timeout) as session:
                async with session.post(self.DDGO_URL, data=payload, headers=headers) as resp:
                    html = await resp.text()

            # Extract result links
            pattern = r'<a class="result__a" href="(https?://[^"]+)"'
            matches = re.findall(pattern, html)

            # Filter low-quality domains
            skip_domains = {"reddit.com", "quora.com", "pinterest.com"}
            for url in matches:
                domain = urlparse(url).netloc.replace("www.", "")
                if domain not in skip_domains and len(urls) < max_results:
                    urls.append(url)
        except Exception:
            pass
        return urls

    async def search(self, query: str, max_results: int = 6) -> list[str]:
        return await self.search_ddg(query, max_results)


class ResearchEngine:
    """Full research pipeline for MICRODRAGON."""

    def __init__(self):
        self.crawler = WebCrawler()
        self.search_engine = SearchEngine()

    async def research(self, query: str, depth: int = 5) -> ResearchResult:
        """Full research pipeline: search → crawl → extract → structure."""

        result = ResearchResult(query=query)

        # 1. Search
        urls = await self.search_engine.search(query, max_results=depth)
        if not urls:
            result.summary = "No search results found."
            return result

        # 2. Crawl
        sources = await self.crawler.fetch_many(urls, max_concurrent=4)

        # 3. Score relevance
        scored = self._score_sources(sources, query)
        result.sources = scored[:depth]

        # 4. Extract key facts (heuristic)
        result.key_facts = self._extract_facts(scored)
        result.confidence = min(len(scored) / depth, 1.0)

        return result

    def _score_sources(self, sources: list[WebSource], query: str) -> list[WebSource]:
        query_words = set(query.lower().split())
        for source in sources:
            text_lower = source.content.lower()
            matches = sum(1 for w in query_words if w in text_lower)
            source.relevance_score = matches / max(len(query_words), 1)

        return sorted(sources, key=lambda s: s.relevance_score, reverse=True)

    def _extract_facts(self, sources: list[WebSource]) -> list[str]:
        """Extract candidate key sentences."""
        facts = []
        for source in sources[:3]:
            sentences = re.split(r'[.!?]+', source.content)
            for sent in sentences:
                sent = sent.strip()
                if 50 < len(sent) < 300 and sent[0].isupper():
                    facts.append(sent)
                if len(facts) >= 10:
                    break
        return facts[:10]

    def format_for_prompt(self, result: ResearchResult) -> str:
        """Format research results as context for the AI brain."""
        lines = [f"## Research: {result.query}\n"]

        lines.append("### Sources\n")
        for i, src in enumerate(result.sources[:5], 1):
            lines.append(f"**[{i}] {src.title}** ({src.domain})")
            # Send first 600 chars of each source
            snippet = src.content[:600].replace("\n", " ").strip()
            lines.append(f"{snippet}\n")

        if result.key_facts:
            lines.append("### Key Facts Extracted\n")
            for fact in result.key_facts[:5]:
                lines.append(f"• {fact}")

        return "\n".join(lines)


# ── Module entry point for IPC ────────────────────────────────────────────────

async def main():
    import sys
    query = " ".join(sys.argv[1:]) if len(sys.argv) > 1 else "MICRODRAGON AI agent"
    print(f"[MICRODRAGON Research] Researching: {query}")
    engine = ResearchEngine()
    result = await engine.research(query)
    print(f"Found {len(result.sources)} sources")
    for src in result.sources[:3]:
        print(f"  • {src.title} — {src.domain} (score: {src.relevance_score:.2f})")


if __name__ == "__main__":
    asyncio.run(main())
