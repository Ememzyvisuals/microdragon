// microdragon-core/src/memory/sqlite.rs
// Persistent conversation memory backed by SQLite
// THIS IS THE MISSING LINK — replaces in-memory Vec with real persistence

use anyhow::{Context, Result};
use chrono::Utc;
use rusqlite::{Connection, params};
use std::path::PathBuf;
use tracing::{info, debug};

use crate::config::providers::{ChatMessage, MessageRole};

/// Persistent SQLite-backed memory store
pub struct PersistentMemory {
    conn: Connection,
    session_id: String,
}

impl PersistentMemory {
    pub fn open(db_path: &PathBuf, session_id: &str) -> Result<Self> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(db_path)
            .with_context(|| format!("Cannot open DB at {:?}", db_path))?;

        // Optimise for interactive latency
        conn.execute_batch("
            PRAGMA journal_mode=WAL;
            PRAGMA synchronous=NORMAL;
            PRAGMA cache_size=10000;
            PRAGMA temp_store=MEMORY;
        ")?;

        // Ensure schema exists
        conn.execute_batch("
            CREATE TABLE IF NOT EXISTS messages (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id  TEXT    NOT NULL,
                role        TEXT    NOT NULL CHECK(role IN ('user','assistant','system')),
                content     TEXT    NOT NULL,
                tokens      INTEGER DEFAULT 0,
                model       TEXT    DEFAULT '',
                source      TEXT    DEFAULT 'cli',
                created_at  TEXT    NOT NULL DEFAULT ''
            );
            CREATE INDEX IF NOT EXISTS idx_msg_session
                ON messages(session_id, id DESC);

            CREATE TABLE IF NOT EXISTS sessions (
                id          TEXT PRIMARY KEY,
                label       TEXT DEFAULT '',
                created_at  TEXT NOT NULL DEFAULT '',
                last_active TEXT NOT NULL DEFAULT '',
                message_count INTEGER DEFAULT 0,
                total_tokens  INTEGER DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS tasks (
                id           TEXT PRIMARY KEY,
                session_id   TEXT NOT NULL,
                task_type    TEXT NOT NULL,
                input        TEXT NOT NULL,
                output       TEXT,
                status       TEXT NOT NULL DEFAULT 'pending',
                agent        TEXT DEFAULT 'master',
                tokens_used  INTEGER DEFAULT 0,
                latency_ms   INTEGER DEFAULT 0,
                feedback     TEXT,
                created_at   TEXT NOT NULL DEFAULT '',
                completed_at TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_tasks_session ON tasks(session_id, created_at DESC);

            CREATE TABLE IF NOT EXISTS feedback_log (
                id         INTEGER PRIMARY KEY AUTOINCREMENT,
                task_id    TEXT NOT NULL,
                score      TEXT NOT NULL CHECK(score IN ('good','bad','neutral')),
                created_at TEXT NOT NULL DEFAULT ''
            );

            CREATE TABLE IF NOT EXISTS code_artifacts (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id  TEXT NOT NULL,
                filename    TEXT NOT NULL,
                language    TEXT NOT NULL,
                content     TEXT NOT NULL,
                description TEXT DEFAULT '',
                created_at  TEXT NOT NULL DEFAULT ''
            );

            CREATE TABLE IF NOT EXISTS improvements (
                id          TEXT PRIMARY KEY,
                observation TEXT NOT NULL,
                suggestion  TEXT NOT NULL,
                applied     INTEGER DEFAULT 0,
                created_at  TEXT NOT NULL DEFAULT ''
            );

            CREATE TABLE IF NOT EXISTS settings (
                key         TEXT PRIMARY KEY,
                value       TEXT NOT NULL,
                updated_at  TEXT NOT NULL DEFAULT ''
            );

            CREATE TABLE IF NOT EXISTS watch_conditions (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                condition   TEXT NOT NULL,
                action      TEXT NOT NULL,
                last_checked TEXT,
                triggered   INTEGER DEFAULT 0,
                enabled     INTEGER DEFAULT 1,
                created_at  TEXT NOT NULL DEFAULT ''
            );
        ")?;

        // Upsert session record
        conn.execute(
            "INSERT OR IGNORE INTO sessions (id) VALUES (?1)",
            params![session_id],
        )?;

        info!("PersistentMemory opened: session={}", session_id);
        Ok(Self { conn, session_id: session_id.to_string() })
    }

    // ── Message storage ───────────────────────────────────────────────────

    pub fn store_message(&self, role: &str, content: &str,
                          tokens: u32, model: &str, source: &str) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO messages (session_id, role, content, tokens, model, source)
             VALUES (?1,?2,?3,?4,?5,?6)",
            params![self.session_id, role, content, tokens, model, source],
        )?;
        let id = self.conn.last_insert_rowid();

        // Update session stats
        self.conn.execute(
            "UPDATE sessions SET
                last_active = strftime('%Y-%m-%dT%H:%M:%SZ','now'),
                message_count = message_count + 1,
                total_tokens = total_tokens + ?1
             WHERE id = ?2",
            params![tokens, self.session_id],
        )?;

        debug!("Stored message id={} role={} tokens={}", id, role, tokens);
        Ok(id)
    }

    pub fn get_recent_messages(&self, limit: usize) -> Result<Vec<ChatMessage>> {
        let mut stmt = self.conn.prepare(
            "SELECT role, content FROM messages
             WHERE session_id = ?1
             ORDER BY id DESC LIMIT ?2"
        )?;

        let rows: Vec<(String, String)> = stmt.query_map(
            params![self.session_id, limit as i64],
            |row| Ok((row.get(0)?, row.get(1)?))
        )?.filter_map(|r| r.ok()).collect();

        let mut messages: Vec<ChatMessage> = rows.into_iter().map(|(role, content)| {
            ChatMessage {
                role: match role.as_str() {
                    "user"      => MessageRole::User,
                    "assistant" => MessageRole::Assistant,
                    _           => MessageRole::System,
                },
                content,
            }
        }).collect();

        messages.reverse(); // chronological order
        Ok(messages)
    }

    pub fn get_all_sessions(&self) -> Result<Vec<SessionSummary>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, label, created_at, last_active, message_count, total_tokens
             FROM sessions ORDER BY last_active DESC LIMIT 100"
        )?;

        let sessions = stmt.query_map([], |row| {
            Ok(SessionSummary {
                id: row.get(0)?,
                label: row.get(1)?,
                created_at: row.get(2)?,
                last_active: row.get(3)?,
                message_count: row.get::<_,i64>(4)? as usize,
                total_tokens: row.get::<_,i64>(5)? as u64,
            })
        })?.filter_map(|r| r.ok()).collect();

        Ok(sessions)
    }

    pub fn clear_session(&self) -> Result<usize> {
        let n = self.conn.execute(
            "DELETE FROM messages WHERE session_id = ?1",
            params![self.session_id],
        )?;
        self.conn.execute(
            "UPDATE sessions SET message_count=0, total_tokens=0 WHERE id=?1",
            params![self.session_id],
        )?;
        Ok(n)
    }

    // ── Task storage ──────────────────────────────────────────────────────

    pub fn save_task(&self, id: &str, task_type: &str, input: &str,
                      output: Option<&str>, status: &str, agent: &str,
                      tokens: u32, latency_ms: u64) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO tasks
             (id, session_id, task_type, input, output, status, agent, tokens_used, latency_ms)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
            params![id, self.session_id, task_type, input, output,
                    status, agent, tokens, latency_ms as i64],
        )?;
        Ok(())
    }

    pub fn record_feedback(&self, task_id: &str, score: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO feedback_log (task_id, score) VALUES (?1,?2)",
            params![task_id, score],
        )?;
        self.conn.execute(
            "UPDATE tasks SET feedback=?1 WHERE id=?2",
            params![score, task_id],
        )?;
        Ok(())
    }

    pub fn get_performance_stats(&self) -> Result<PerformanceStats> {
        let total: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM tasks WHERE session_id=?1", params![self.session_id], |r| r.get(0)
        ).unwrap_or(0);

        let successful: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM tasks WHERE session_id=?1 AND status='complete'",
            params![self.session_id], |r| r.get(0)
        ).unwrap_or(0);

        let avg_latency: f64 = self.conn.query_row(
            "SELECT AVG(latency_ms) FROM tasks WHERE session_id=?1",
            params![self.session_id], |r| r.get::<_,f64>(0)
        ).unwrap_or(0.0);

        let total_tokens: i64 = self.conn.query_row(
            "SELECT COALESCE(SUM(tokens_used),0) FROM tasks WHERE session_id=?1",
            params![self.session_id], |r| r.get(0)
        ).unwrap_or(0);

        let good_feedback: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM feedback_log f
             JOIN tasks t ON f.task_id=t.id
             WHERE t.session_id=?1 AND f.score='good'",
            params![self.session_id], |r| r.get(0)
        ).unwrap_or(0);

        let bad_feedback: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM feedback_log f
             JOIN tasks t ON f.task_id=t.id
             WHERE t.session_id=?1 AND f.score='bad'",
            params![self.session_id], |r| r.get(0)
        ).unwrap_or(0);

        Ok(PerformanceStats {
            total_tasks: total as u64,
            successful: successful as u64,
            failed: (total - successful) as u64,
            avg_latency_ms: avg_latency as u64,
            total_tokens: total_tokens as u64,
            success_rate: if total > 0 { successful as f64 / total as f64 * 100.0 } else { 0.0 },
            good_feedback: good_feedback as u64,
            bad_feedback: bad_feedback as u64,
        })
    }

    // ── Code artifacts ────────────────────────────────────────────────────

    pub fn save_code_artifact(&self, filename: &str, language: &str,
                               content: &str, description: &str) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO code_artifacts (session_id, filename, language, content, description)
             VALUES (?1,?2,?3,?4,?5)",
            params![self.session_id, filename, language, content, description],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    // ── Watch conditions ──────────────────────────────────────────────────

    pub fn add_watch_condition(&self, condition: &str, action: &str) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO watch_conditions (condition, action) VALUES (?1,?2)",
            params![condition, action],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_watch_conditions(&self) -> Result<Vec<WatchCondition>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, condition, action, enabled FROM watch_conditions WHERE enabled=1"
        )?;
        let conditions = stmt.query_map([], |row| {
            Ok(WatchCondition {
                id: row.get(0)?,
                condition: row.get(1)?,
                action: row.get(2)?,
                enabled: row.get::<_,i64>(3)? == 1,
            })
        })?.filter_map(|r| r.ok()).collect();
        Ok(conditions)
    }

    // ── Settings ──────────────────────────────────────────────────────────

    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1,?2)",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn get_setting(&self, key: &str) -> Result<Option<String>> {
        match self.conn.query_row(
            "SELECT value FROM settings WHERE key=?1", params![key], |r| r.get(0)
        ) {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn db_size_kb(&self) -> u64 {
        let pages: i64 = self.conn.query_row("PRAGMA page_count", [], |r| r.get(0)).unwrap_or(0);
        let page_size: i64 = self.conn.query_row("PRAGMA page_size", [], |r| r.get(0)).unwrap_or(4096);
        (pages * page_size / 1024) as u64
    }
}

// ── Data types ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct SessionSummary {
    pub id: String,
    pub label: String,
    pub created_at: String,
    pub last_active: String,
    pub message_count: usize,
    pub total_tokens: u64,
}

#[derive(Debug, Clone)]
pub struct PerformanceStats {
    pub total_tasks: u64,
    pub successful: u64,
    pub failed: u64,
    pub avg_latency_ms: u64,
    pub total_tokens: u64,
    pub success_rate: f64,
    pub good_feedback: u64,
    pub bad_feedback: u64,
}

#[derive(Debug, Clone)]
pub struct WatchCondition {
    pub id: i64,
    pub condition: String,
    pub action: String,
    pub enabled: bool,
}
