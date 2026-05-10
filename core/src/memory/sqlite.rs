// memory/sqlite.rs — tokio-rusqlite: fully Send+Sync, no E0277 errors ever
use anyhow::Result;
use std::path::PathBuf;
use tokio_rusqlite::Connection;
use tracing::info;
use crate::config::providers::{ChatMessage, MessageRole};

/// Cloneable handle — Connection lives on its own background thread (tokio-rusqlite).
/// This makes PersistentMemory: Send + Sync, fixing ALL axum/tokio::spawn errors.
#[derive(Clone)]
pub struct PersistentMemory {
    conn:       Connection,
    session_id: String,
}

const SCHEMA: &str = "
PRAGMA journal_mode=WAL;
PRAGMA synchronous=NORMAL;
PRAGMA cache_size=10000;
PRAGMA temp_store=MEMORY;
CREATE TABLE IF NOT EXISTS messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id TEXT NOT NULL, role TEXT NOT NULL, content TEXT NOT NULL,
    tokens INTEGER DEFAULT 0, model TEXT DEFAULT '', source TEXT DEFAULT 'cli',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_msg_session ON messages(session_id, id DESC);
CREATE TABLE IF NOT EXISTS tasks (
    id TEXT PRIMARY KEY, kind TEXT NOT NULL, input TEXT NOT NULL, output TEXT,
    status TEXT NOT NULL, agent TEXT NOT NULL,
    tokens INTEGER DEFAULT 0, latency_ms INTEGER DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE TABLE IF NOT EXISTS feedback (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    task_id TEXT NOT NULL, score TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE TABLE IF NOT EXISTS code_artifacts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    filename TEXT NOT NULL, language TEXT NOT NULL,
    content TEXT NOT NULL, description TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE TABLE IF NOT EXISTS watch_conditions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    condition TEXT NOT NULL, action TEXT NOT NULL, enabled INTEGER DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
";

impl PersistentMemory {
    pub async fn open(db_path: &PathBuf, session_id: &str) -> Result<Self> {
        if let Some(p) = db_path.parent() { std::fs::create_dir_all(p)?; }
        let conn = Connection::open(db_path).await?;
        conn.call(|c| { c.execute_batch(SCHEMA)?; Ok(()) }).await?;
        info!("DB ready: {:?}", db_path);
        Ok(Self { conn, session_id: session_id.to_string() })
    }

    pub async fn store_message_async(&self, role: &str, content: &str,
                                     tokens: u32, model: &str, source: &str) -> Result<()> {
        let (sid,r,ct,t,m,s) = (self.session_id.clone(), role.to_string(),
            content.to_string(), tokens, model.to_string(), source.to_string());
        self.conn.call(move |c| {
            c.execute("INSERT INTO messages (session_id,role,content,tokens,model,source) VALUES (?1,?2,?3,?4,?5,?6)",
                rusqlite::params![sid,r,ct,t,m,s])?; Ok(())
        }).await.map_err(|e| anyhow::anyhow!(e))
    }

    pub fn store_message(&self, role: &str, content: &str,
                         tokens: u32, model: &str, source: &str) -> Result<()> {
        let rt = tokio::runtime::Handle::try_current();
        match rt {
            Ok(handle) => {
                tokio::task::block_in_place(|| {
                    handle.block_on(self.store_message_async(role, content, tokens, model, source))
                })
            }
            Err(_) => {
                let rt2 = tokio::runtime::Builder::new_current_thread().enable_all().build()?;
                rt2.block_on(self.store_message_async(role, content, tokens, model, source))
            }
        }
    }

    pub async fn save_task_async(&self, id: &str, kind: &str, input: &str,
                                  output: Option<&str>, status: &str, agent: &str,
                                  tokens: u32, latency_ms: u64) -> Result<()> {
        let (id,kind,inp,out,stat,ag) = (id.to_string(),kind.to_string(),input.to_string(),
            output.map(|s|s.to_string()),status.to_string(),agent.to_string());
        self.conn.call(move |c| {
            c.execute("INSERT OR REPLACE INTO tasks (id,kind,input,output,status,agent,tokens,latency_ms) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
                rusqlite::params![id,kind,inp,out,stat,ag,tokens,latency_ms])?; Ok(())
        }).await.map_err(|e| anyhow::anyhow!(e))
    }

    pub fn save_task(&self, id: &str, kind: &str, input: &str,
                     output: Option<&str>, status: &str, agent: &str,
                     tokens: u32, latency_ms: u64) -> Result<()> {
        let rt = tokio::runtime::Handle::try_current();
        match rt {
            Ok(handle) => tokio::task::block_in_place(|| {
                handle.block_on(self.save_task_async(id,kind,input,output,status,agent,tokens,latency_ms))
            }),
            Err(_) => {
                let rt2 = tokio::runtime::Builder::new_current_thread().enable_all().build()?;
                rt2.block_on(self.save_task_async(id,kind,input,output,status,agent,tokens,latency_ms))
            }
        }
    }

    pub fn record_feedback(&self, task_id: &str, score: &str) -> Result<()> {
        let (tid,sc) = (task_id.to_string(), score.to_string());
        let conn = self.conn.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                conn.call(move |c| {
                    c.execute("INSERT INTO feedback (task_id,score) VALUES (?1,?2)",
                        rusqlite::params![tid,sc])?; Ok(())
                }).await.map_err(|e| anyhow::anyhow!(e))
            })
        })
    }

    pub fn clear_session(&self) -> Result<()> {
        let sid = self.session_id.clone();
        let conn = self.conn.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                conn.call(move |c| {
                    c.execute("DELETE FROM messages WHERE session_id=?1", rusqlite::params![sid])?; Ok(())
                }).await.map_err(|e| anyhow::anyhow!(e))
            })
        })
    }

    pub fn get_recent_messages(&self, n: usize) -> Result<Vec<ChatMessage>> {
        let sid = self.session_id.clone();
        let conn = self.conn.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                conn.call(move |c| {
                    let mut stmt = c.prepare(
                        "SELECT role,content FROM messages WHERE session_id=?1 ORDER BY id DESC LIMIT ?2")?;
                    let mut msgs: Vec<ChatMessage> = stmt.query_map(
                        rusqlite::params![sid, n as i64],
                        |row| Ok((row.get::<_,String>(0)?, row.get::<_,String>(1)?))
                    )?.filter_map(|r| r.ok()).map(|(role,content)| ChatMessage {
                        role: if role=="user" { MessageRole::User } else { MessageRole::Assistant },
                        content,
                    }).collect();
                    msgs.reverse();
                    Ok(msgs)
                }).await.map_err(|e| anyhow::anyhow!(e))
            })
        })
    }

    pub fn get_performance_stats(&self) -> Result<PerformanceStats> {
        let conn = self.conn.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                conn.call(|c| {
                    let total: i64 = c.query_row("SELECT COUNT(*) FROM tasks",[], |r| r.get(0))?;
                    let avg: f64 = c.query_row("SELECT COALESCE(AVG(latency_ms),0.0) FROM tasks",[], |r| r.get(0))?;
                    Ok(PerformanceStats { total_tasks: total as u64, avg_latency_ms: avg })
                }).await.map_err(|e| anyhow::anyhow!(e))
            })
        })
    }

    pub fn save_code_artifact(&self, filename: &str, language: &str,
                               content: &str, description: &str) -> Result<i64> {
        let (fn_,lang,ct,desc) = (filename.to_string(),language.to_string(),
                                   content.to_string(),description.to_string());
        let conn = self.conn.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                conn.call(move |c| {
                    c.execute("INSERT INTO code_artifacts (filename,language,content,description) VALUES (?1,?2,?3,?4)",
                        rusqlite::params![fn_,lang,ct,desc])?;
                    Ok(c.last_insert_rowid())
                }).await.map_err(|e| anyhow::anyhow!(e))
            })
        })
    }

    pub fn add_watch_condition(&self, condition: &str, action: &str) -> Result<i64> {
        let (cond,act) = (condition.to_string(), action.to_string());
        let conn = self.conn.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                conn.call(move |c| {
                    c.execute("INSERT INTO watch_conditions (condition,action) VALUES (?1,?2)",
                        rusqlite::params![cond,act])?;
                    Ok(c.last_insert_rowid())
                }).await.map_err(|e| anyhow::anyhow!(e))
            })
        })
    }

    pub fn get_watch_conditions(&self) -> Result<Vec<WatchCondition>> {
        let conn = self.conn.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                conn.call(|c| {
                    let mut stmt = c.prepare(
                        "SELECT id,condition,action FROM watch_conditions WHERE enabled=1")?;
                    let rows = stmt.query_map([], |row| Ok(WatchCondition {
                        id: row.get(0)?, condition: row.get(1)?, action: row.get(2)?
                    }))?;
                    Ok(rows.filter_map(|r| r.ok()).collect())
                }).await.map_err(|e| anyhow::anyhow!(e))
            })
        })
    }

    pub fn db_size_kb(&self) -> u64 {
        let conn = self.conn.clone();
        match tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                conn.call(|c| {
                    let n: i64 = c.query_row(
                        "SELECT page_count*page_size/1024 FROM pragma_page_count(),pragma_page_size()",
                        [], |r| r.get(0)).unwrap_or(0);
                    Ok(n as u64)
                }).await
            })
        }) {
            Ok(n) => n,
            Err(_) => 0,
        }
    }
}

#[derive(Debug,Clone,Default)]
pub struct PerformanceStats { pub total_tasks: u64, pub avg_latency_ms: f64 }

#[derive(Debug,Clone)]
pub struct WatchCondition { pub id: i64, pub condition: String, pub action: String }
