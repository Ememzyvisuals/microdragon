// microdragon-core/src/persistence/mod.rs — full SQLite persistence layer

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use rusqlite::{Connection, params};

/// Persistent conversation message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredMessage {
    pub id: i64,
    pub session_id: String,
    pub role: String,
    pub content: String,
    pub tokens: u32,
    pub model: String,
    pub created_at: DateTime<Utc>,
}

/// Stored task record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredTask {
    pub id: String,
    pub task_type: String,
    pub input: String,
    pub output: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub tokens_used: u32,
    pub latency_ms: u64,
}

pub struct Database {
    conn: Connection,
    path: PathBuf,
}

impl Database {
    pub fn open(path: &PathBuf) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path)
            .with_context(|| format!("Failed to open DB at {:?}", path))?;

        // Enable WAL mode for better concurrent performance
        conn.execute_batch("
            PRAGMA journal_mode=WAL;
            PRAGMA synchronous=NORMAL;
            PRAGMA foreign_keys=ON;
            PRAGMA temp_store=MEMORY;
        ")?;

        let db = Self { conn, path: path.clone() };
        db.migrate()?;
        Ok(db)
    }

    fn migrate(&self) -> Result<()> {
        self.conn.execute_batch("
            CREATE TABLE IF NOT EXISTS messages (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id  TEXT    NOT NULL,
                role        TEXT    NOT NULL,
                content     TEXT    NOT NULL,
                tokens      INTEGER DEFAULT 0,
                model       TEXT    DEFAULT '',
                created_at  TEXT    NOT NULL DEFAULT (datetime('now'))
            );

            CREATE INDEX IF NOT EXISTS idx_messages_session
                ON messages(session_id, created_at DESC);

            CREATE TABLE IF NOT EXISTS tasks (
                id           TEXT PRIMARY KEY,
                task_type    TEXT NOT NULL,
                input        TEXT NOT NULL,
                output       TEXT,
                status       TEXT NOT NULL DEFAULT 'pending',
                created_at   TEXT NOT NULL DEFAULT (datetime('now')),
                completed_at TEXT,
                tokens_used  INTEGER DEFAULT 0,
                latency_ms   INTEGER DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS settings (
                key   TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS code_artifacts (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                filename    TEXT NOT NULL,
                language    TEXT NOT NULL,
                content     TEXT NOT NULL,
                description TEXT DEFAULT '',
                task_id     TEXT,
                created_at  TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS research_cache (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                query       TEXT NOT NULL,
                result_json TEXT NOT NULL,
                source_urls TEXT NOT NULL,
                created_at  TEXT NOT NULL DEFAULT (datetime('now')),
                expires_at  TEXT
            );

            CREATE INDEX IF NOT EXISTS idx_research_query
                ON research_cache(query);
        ")?;
        Ok(())
    }

    // ── Messages ─────────────────────────────────────────────────────────

    pub fn insert_message(&self, session_id: &str, role: &str, content: &str,
                           tokens: u32, model: &str) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO messages (session_id, role, content, tokens, model) VALUES (?1,?2,?3,?4,?5)",
            params![session_id, role, content, tokens, model],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_messages(&self, session_id: &str, limit: usize) -> Result<Vec<StoredMessage>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, session_id, role, content, tokens, model, created_at
             FROM messages WHERE session_id = ?1
             ORDER BY created_at DESC LIMIT ?2"
        )?;

        let rows = stmt.query_map(params![session_id, limit as i64], |row| {
            Ok(StoredMessage {
                id: row.get(0)?,
                session_id: row.get(1)?,
                role: row.get(2)?,
                content: row.get(3)?,
                tokens: row.get::<_, i64>(4)? as u32,
                model: row.get(5)?,
                created_at: Utc::now(), // simplified
            })
        })?;

        let mut messages: Vec<StoredMessage> = rows.filter_map(|r| r.ok()).collect();
        messages.reverse(); // oldest first
        Ok(messages)
    }

    pub fn clear_messages(&self, session_id: &str) -> Result<usize> {
        let n = self.conn.execute(
            "DELETE FROM messages WHERE session_id = ?1",
            params![session_id]
        )?;
        Ok(n)
    }

    // ── Tasks ─────────────────────────────────────────────────────────────

    pub fn upsert_task(&self, task: &StoredTask) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO tasks (id, task_type, input, output, status, tokens_used, latency_ms)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                task.id, task.task_type, task.input,
                task.output, task.status,
                task.tokens_used, task.latency_ms
            ],
        )?;
        Ok(())
    }

    pub fn get_recent_tasks(&self, limit: usize) -> Result<Vec<StoredTask>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, task_type, input, output, status, created_at, completed_at, tokens_used, latency_ms
             FROM tasks ORDER BY created_at DESC LIMIT ?1"
        )?;

        let rows = stmt.query_map(params![limit as i64], |row| {
            Ok(StoredTask {
                id: row.get(0)?,
                task_type: row.get(1)?,
                input: row.get(2)?,
                output: row.get(3)?,
                status: row.get(4)?,
                created_at: Utc::now(),
                completed_at: None,
                tokens_used: row.get::<_, i64>(7)? as u32,
                latency_ms: row.get::<_, i64>(8)? as u64,
            })
        })?;

        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    // ── Settings ─────────────────────────────────────────────────────────

    pub fn set(&self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO settings (key, value, updated_at)
             VALUES (?1, ?2, datetime('now'))",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn get(&self, key: &str) -> Result<Option<String>> {
        let result = self.conn.query_row(
            "SELECT value FROM settings WHERE key = ?1",
            params![key],
            |row| row.get::<_, String>(0),
        );
        match result {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    // ── Code artifacts ────────────────────────────────────────────────────

    pub fn save_code(&self, filename: &str, language: &str, content: &str,
                     description: &str) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO code_artifacts (filename, language, content, description)
             VALUES (?1, ?2, ?3, ?4)",
            params![filename, language, content, description],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    // ── Stats ─────────────────────────────────────────────────────────────

    pub fn stats(&self) -> Result<DbStats> {
        let msg_count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM messages", [], |r| r.get(0)
        ).unwrap_or(0);

        let task_count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM tasks", [], |r| r.get(0)
        ).unwrap_or(0);

        let total_tokens: i64 = self.conn.query_row(
            "SELECT COALESCE(SUM(tokens),0) FROM messages", [], |r| r.get(0)
        ).unwrap_or(0);

        let code_count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM code_artifacts", [], |r| r.get(0)
        ).unwrap_or(0);

        Ok(DbStats {
            messages: msg_count as usize,
            tasks: task_count as usize,
            total_tokens: total_tokens as u64,
            code_artifacts: code_count as usize,
            db_path: self.path.display().to_string(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbStats {
    pub messages: usize,
    pub tasks: usize,
    pub total_tokens: u64,
    pub code_artifacts: usize,
    pub db_path: String,
}
