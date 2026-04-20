// microdragon-core/src/memory/vector.rs
// Local semantic vector memory — search past conversations by meaning, not keyword
// Uses cosine similarity on embeddings stored in SQLite as JSON blobs
// Embedding model: Ollama nomic-embed-text (free, local) or API fallback

use anyhow::Result;
use rusqlite::{Connection, params};
use std::path::PathBuf;
use tracing::{debug, warn};

/// A stored memory chunk with its embedding
#[derive(Debug, Clone)]
pub struct MemoryChunk {
    pub id: i64,
    pub content: String,
    pub embedding: Vec<f32>,
    pub source: String,
    pub created_at: String,
    pub relevance: f32,
}

pub struct VectorMemory {
    conn: Connection,
    embedder: EmbeddingBackend,
    dimension: usize,
}

enum EmbeddingBackend {
    Ollama { endpoint: String, model: String },
    OpenAI { api_key: String },
    Stub, // fallback: keyword-based fake embeddings
}

impl VectorMemory {
    pub async fn new(db_path: &PathBuf) -> Result<Self> {
        if let Some(p) = db_path.parent() { std::fs::create_dir_all(p)?; }

        let conn = Connection::open(db_path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;
        conn.execute_batch("
            CREATE TABLE IF NOT EXISTS memory_chunks (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                content     TEXT NOT NULL,
                embedding   BLOB NOT NULL,
                source      TEXT DEFAULT 'conversation',
                session_id  TEXT DEFAULT '',
                created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
            );
            CREATE INDEX IF NOT EXISTS idx_chunks_session ON memory_chunks(session_id);
        ")?;

        // Detect available embedding backend
        let embedder = Self::detect_backend().await;
        let dimension = match &embedder {
            EmbeddingBackend::Ollama { .. } => 768,
            EmbeddingBackend::OpenAI { .. } => 1536,
            EmbeddingBackend::Stub => 64,
        };

        Ok(Self { conn, embedder, dimension })
    }

    async fn detect_backend() -> EmbeddingBackend {
        // 1. Try local Ollama (free, no API cost)
        if let Ok(resp) = reqwest::get("http://localhost:11434/api/tags").await {
            if resp.status().is_success() {
                debug!("VectorMemory: using Ollama nomic-embed-text");
                return EmbeddingBackend::Ollama {
                    endpoint: "http://localhost:11434".to_string(),
                    model: "nomic-embed-text".to_string(),
                };
            }
        }

        // 2. Try OpenAI if key available
        if let Ok(key) = std::env::var("OPENAI_API_KEY") {
            if !key.is_empty() {
                debug!("VectorMemory: using OpenAI text-embedding-3-small");
                return EmbeddingBackend::OpenAI { api_key: key };
            }
        }

        // 3. Fallback stub
        warn!("VectorMemory: no embedding backend found, using keyword stub. Install Ollama for semantic memory.");
        EmbeddingBackend::Stub
    }

    /// Store a text chunk with its embedding
    pub async fn store(&self, content: &str, source: &str, session_id: &str) -> Result<i64> {
        let embedding = self.embed(content).await?;
        let blob = Self::vec_to_blob(&embedding);

        self.conn.execute(
            "INSERT INTO memory_chunks (content, embedding, source, session_id)
             VALUES (?1,?2,?3,?4)",
            params![content, blob, source, session_id],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Search for semantically similar memories
    pub async fn search(&self, query: &str, top_k: usize) -> Result<Vec<MemoryChunk>> {
        let query_embedding = self.embed(query).await?;

        // Load all chunks and compute cosine similarity
        let mut stmt = self.conn.prepare(
            "SELECT id, content, embedding, source, created_at FROM memory_chunks
             ORDER BY id DESC LIMIT 1000"
        )?;

        let mut candidates: Vec<MemoryChunk> = stmt.query_map([], |row| {
            let blob: Vec<u8> = row.get(2)?;
            let embedding = Self::blob_to_vec(&blob);
            Ok(MemoryChunk {
                id: row.get(0)?,
                content: row.get(1)?,
                embedding,
                source: row.get(3)?,
                created_at: row.get(4)?,
                relevance: 0.0,
            })
        })?.filter_map(|r| r.ok()).collect();

        // Score by cosine similarity
        for chunk in &mut candidates {
            chunk.relevance = Self::cosine_similarity(&query_embedding, &chunk.embedding);
        }

        // Sort by relevance, take top_k
        candidates.sort_by(|a, b| b.relevance.partial_cmp(&a.relevance).unwrap());
        candidates.truncate(top_k);

        Ok(candidates)
    }

    /// Generate embedding for text
    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        match &self.embedder {
            EmbeddingBackend::Ollama { endpoint, model } => {
                self.embed_ollama(endpoint, model, text).await
            }
            EmbeddingBackend::OpenAI { api_key } => {
                self.embed_openai(api_key, text).await
            }
            EmbeddingBackend::Stub => {
                Ok(Self::stub_embedding(text, self.dimension))
            }
        }
    }

    async fn embed_ollama(&self, endpoint: &str, model: &str, text: &str) -> Result<Vec<f32>> {
        let client = reqwest::Client::new();
        let resp = client.post(format!("{}/api/embeddings", endpoint))
            .json(&serde_json::json!({ "model": model, "prompt": text }))
            .timeout(std::time::Duration::from_secs(10))
            .send().await?;

        let json: serde_json::Value = resp.json().await?;
        let embedding: Vec<f32> = json["embedding"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|v| v.as_f64().map(|f| f as f32))
            .collect();

        Ok(embedding)
    }

    async fn embed_openai(&self, api_key: &str, text: &str) -> Result<Vec<f32>> {
        let client = reqwest::Client::new();
        let resp = client.post("https://api.openai.com/v1/embeddings")
            .bearer_auth(api_key)
            .json(&serde_json::json!({
                "model": "text-embedding-3-small",
                "input": text
            }))
            .timeout(std::time::Duration::from_secs(10))
            .send().await?;

        let json: serde_json::Value = resp.json().await?;
        let embedding: Vec<f32> = json["data"][0]["embedding"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|v| v.as_f64().map(|f| f as f32))
            .collect();

        Ok(embedding)
    }

    /// Keyword-based stub embedding (no model needed)
    fn stub_embedding(text: &str, dim: usize) -> Vec<f32> {
        let mut vec = vec![0.0f32; dim];
        for (i, byte) in text.bytes().enumerate() {
            vec[i % dim] += byte as f32 / 255.0;
        }
        // L2 normalize
        let norm: f32 = vec.iter().map(|x| x * x).sum::<f32>().sqrt().max(1e-8);
        vec.iter_mut().for_each(|x| *x /= norm);
        vec
    }

    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.is_empty() || b.is_empty() || a.len() != b.len() { return 0.0; }
        let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm_a == 0.0 || norm_b == 0.0 { return 0.0; }
        dot / (norm_a * norm_b)
    }

    fn vec_to_blob(vec: &[f32]) -> Vec<u8> {
        vec.iter().flat_map(|f| f.to_le_bytes()).collect()
    }

    fn blob_to_vec(blob: &[u8]) -> Vec<f32> {
        blob.chunks_exact(4)
            .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            .collect()
    }

    pub fn chunk_count(&self) -> usize {
        self.conn.query_row("SELECT COUNT(*) FROM memory_chunks", [], |r| r.get::<_,i64>(0))
            .unwrap_or(0) as usize
    }
}
