// microdragon-core/src/tools/web_search.rs
// Real web search for MICRODRAGON research commands.
//
// Priority:
//   1. Brave Search API  (BRAVE_SEARCH_API_KEY env var) — best quality
//   2. DuckDuckGo HTML   (no key needed)               — reliable fallback
//
// © 2026 EMEMZYVISUALS DIGITALS — Emmanuel Ariyo

use anyhow::{Result, Context};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub title:   String,
    pub url:     String,
    pub snippet: String,
}

#[derive(Debug, Clone)]
pub struct WebSearcher {
    client: Client,
}

impl WebSearcher {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(15))
            .user_agent("Mozilla/5.0 (compatible; Microdragon/0.1; +https://github.com/Ememzyvisuals/microdragon)")
            .build()
            .expect("failed to build web search client");
        Self { client }
    }

    /// Search the web. Returns up to `limit` results.
    pub async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        // Brave if key is set, otherwise DuckDuckGo
        if let Ok(key) = std::env::var("BRAVE_SEARCH_API_KEY") {
            self.brave_search(query, &key, limit).await
        } else {
            self.ddg_search(query, limit).await
        }
    }

    // ─── Brave Search ────────────────────────────────────────────────────────

    async fn brave_search(&self, query: &str, api_key: &str, limit: usize) -> Result<Vec<SearchResult>> {
        let url = format!(
            "https://api.search.brave.com/res/v1/web/search?q={}&count={}",
            urlencoding::encode(query),
            limit.min(10)
        );

        let resp = self.client
            .get(&url)
            .header("Accept", "application/json")
            .header("Accept-Encoding", "gzip")
            .header("X-Subscription-Token", api_key)
            .send()
            .await
            .context("Brave Search request failed")?;

        if !resp.status().is_success() {
            // Fallback to DDG on auth/quota errors
            return self.ddg_search(query, limit).await;
        }

        let json: serde_json::Value = resp.json().await.context("Failed to parse Brave response")?;

        let mut results = Vec::new();
        if let Some(web) = json["web"]["results"].as_array() {
            for item in web.iter().take(limit) {
                results.push(SearchResult {
                    title:   item["title"].as_str().unwrap_or("").to_string(),
                    url:     item["url"].as_str().unwrap_or("").to_string(),
                    snippet: item["description"].as_str().unwrap_or("").to_string(),
                });
            }
        }

        if results.is_empty() {
            return self.ddg_search(query, limit).await;
        }

        Ok(results)
    }

    // ─── DuckDuckGo HTML fallback ─────────────────────────────────────────────

    async fn ddg_search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        // DDG HTML search endpoint
        let url = format!(
            "https://html.duckduckgo.com/html/?q={}&kl=us-en",
            urlencoding::encode(query)
        );

        let html = self.client
            .get(&url)
            .header("Accept", "text/html")
            .send()
            .await
            .context("DuckDuckGo request failed")?
            .text()
            .await
            .context("Failed to read DuckDuckGo response")?;

        Ok(Self::parse_ddg_html(&html, limit))
    }

    fn parse_ddg_html(html: &str, limit: usize) -> Vec<SearchResult> {
        let mut results = Vec::new();

        // DuckDuckGo HTML structure:
        //   <a class="result__a" href="...">Title</a>
        //   <a class="result__snippet">Snippet</a>
        let title_re   = regex::Regex::new(r#"class="result__a"[^>]*href="([^"]*)"[^>]*>([^<]*)</a>"#).unwrap();
        let snippet_re = regex::Regex::new(r#"class="result__snippet"[^>]*>([^<]*(?:<b>[^<]*</b>[^<]*)*)"#).unwrap();

        let titles:   Vec<_> = title_re.captures_iter(html).collect();
        let snippets: Vec<_> = snippet_re.captures_iter(html).collect();

        for (i, cap) in titles.iter().enumerate().take(limit) {
            let url     = cap.get(1).map(|m| m.as_str()).unwrap_or("").to_string();
            let title   = cap.get(2).map(|m| m.as_str()).unwrap_or("").trim().to_string();
            let snippet = snippets.get(i)
                .and_then(|s| s.get(1))
                .map(|m| strip_html_tags(m.as_str()))
                .unwrap_or_default();

            if !title.is_empty() && !url.is_empty() {
                results.push(SearchResult { title, url, snippet });
            }
        }

        results
    }
}

fn strip_html_tags(s: &str) -> String {
    let re = regex::Regex::new(r"<[^>]+>").unwrap();
    re.replace_all(s, "").trim().to_string()
}

/// Format search results into a context block for the AI
pub fn format_for_ai(query: &str, results: &[SearchResult]) -> String {
    if results.is_empty() {
        return format!(
            "No web search results found for: \"{}\". Answer from your training knowledge.",
            query
        );
    }

    let mut ctx = format!(
        "## Web Search Results for: \"{}\"\n\n",
        query
    );

    for (i, r) in results.iter().enumerate() {
        ctx.push_str(&format!(
            "**[{}] {}**\nURL: {}\n{}\n\n",
            i + 1, r.title, r.url, r.snippet
        ));
    }

    ctx.push_str(
        "\nUsing the search results above, provide a comprehensive, accurate answer. \
         Cite sources by number [1], [2], etc. Lead with the most important finding."
    );

    ctx
}
