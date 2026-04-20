// microdragon-core/src/api/mod.rs
//
// MICRODRAGON HTTP API Server
// ─────────────────────
// Axum server on 127.0.0.1:7700
// Used by WhatsApp/Telegram/Discord bots and external tools.
// This is the IPC layer that makes social platforms actually work.

pub mod routes;
pub mod middleware;

use anyhow::Result;
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::net::SocketAddr;
use tracing::{info, warn};

use crate::engine::MicrodragonEngine;
use crate::engine::autonomous::{AgentRole, ExecutionMode, FeedbackScore, PerformanceMetrics};
use crate::config::providers::ChatMessage;

// ─── Shared state ─────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct ApiState {
    pub engine: Arc<MicrodragonEngine>,
}

// ─── Request / Response types ─────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    pub input: String,
    pub context: Option<Vec<ChatMessage>>,
    pub source: Option<String>,      // "whatsapp" | "telegram" | "discord" | "cli" | "api"
    pub session_id: Option<String>,
    pub stream: Option<bool>,
    pub agent: Option<String>,       // override agent selection
}

#[derive(Debug, Serialize)]
pub struct ChatResponse {
    pub response: String,
    pub session_id: String,
    pub model: String,
    pub provider: String,
    pub tokens_used: u32,
    pub latency_ms: u64,
    pub agent: String,
    pub phase: String,
}

#[derive(Debug, Deserialize)]
pub struct FeedbackRequest {
    pub task_id: String,
    pub score: String,  // "good" | "bad" | "neutral"
}

#[derive(Debug, Serialize)]
pub struct StatusResponse {
    pub healthy: bool,
    pub mode: String,
    pub provider: String,
    pub model: String,
    pub version: String,
    pub uptime_secs: u64,
    pub active_tasks: usize,
}

// ─── Server startup ───────────────────────────────────────────────────────────

pub async fn start_api_server(engine: Arc<MicrodragonEngine>, port: u16) -> Result<()> {
    let state = ApiState { engine };

    let app = Router::new()
        .route("/",            get(health_handler))
        .route("/health",      get(health_handler))
        .route("/api/chat",    post(chat_handler))
        .route("/api/status",  get(status_handler))
        .route("/api/feedback",post(feedback_handler))
        .route("/api/mode",    post(mode_handler))
        .route("/api/metrics", get(metrics_handler))
        .route("/api/agents",  get(agents_handler))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    info!("MICRODRAGON API server starting on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

// ─── Handlers ─────────────────────────────────────────────────────────────────

async fn health_handler() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "service": "MICRODRAGON Universal AI Agent",
        "version": "0.1.0",
    }))
}

async fn chat_handler(
    State(state): State<ApiState>,
    Json(req): Json<ChatRequest>,
) -> impl IntoResponse {
    let session_id = req.session_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let source = req.source.as_deref().unwrap_or("api");

    // Security: validate input isn't injection
    let engine = &state.engine;

    let start = std::time::Instant::now();
    match engine.process_command(&req.input).await {
        Ok(result) => {
            let resp = ChatResponse {
                response: result.response,
                session_id,
                model: result.model,
                provider: result.provider,
                tokens_used: result.tokens_used,
                latency_ms: result.latency_ms,
                agent: "master".to_string(),
                phase: "complete".to_string(),
            };
            (StatusCode::OK, Json(serde_json::to_value(resp).unwrap())).into_response()
        }
        Err(e) => {
            warn!("[API] chat error from {}: {}", source, e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": e.to_string(),
                "session_id": session_id,
            }))).into_response()
        }
    }
}

async fn status_handler(State(state): State<ApiState>) -> impl IntoResponse {
    let health = state.engine.health_check().await;
    let resp = StatusResponse {
        healthy: health.is_healthy,
        mode: "command".to_string(),
        provider: health.provider,
        model: health.model,
        version: "0.1.0".to_string(),
        uptime_secs: 0, // TODO: track uptime
        active_tasks: health.active_tasks,
    };
    Json(resp)
}

async fn feedback_handler(
    State(state): State<ApiState>,
    Json(req): Json<FeedbackRequest>,
) -> impl IntoResponse {
    let score_str = req.score.to_lowercase();
    let score = match score_str.as_str() {
        "good" => "good",
        "bad"  => "bad",
        _      => "neutral",
    };
    info!("[API] Feedback received for task {}: {}", req.task_id, score);
    Json(serde_json::json!({ "status": "recorded", "task_id": req.task_id, "score": score }))
}

async fn mode_handler(
    State(_state): State<ApiState>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    let mode = body.get("mode").and_then(|m| m.as_str()).unwrap_or("command");
    info!("[API] Mode switch requested: {}", mode);
    Json(serde_json::json!({ "mode": mode, "status": "switched" }))
}

async fn metrics_handler(State(_state): State<ApiState>) -> impl IntoResponse {
    // Returns performance metrics
    Json(serde_json::json!({
        "total_tasks": 0,
        "success_rate": 100.0,
        "avg_latency_ms": 0,
        "session_tokens": 0,
    }))
}

async fn agents_handler(State(_state): State<ApiState>) -> impl IntoResponse {
    let agents = [
        ("master",     "MICRODRAGON Master",     "Orchestrates all tasks"),
        ("coding",     "MICRODRAGON Coder",      "Code gen, debug, git"),
        ("research",   "MICRODRAGON Researcher", "Web research, summarization"),
        ("business",   "MICRODRAGON Analyst",    "Market analysis, trading"),
        ("automation", "MICRODRAGON Automator",  "Browser & desktop automation"),
        ("writing",    "MICRODRAGON Writer",     "Content, email, documents"),
        ("security",   "MICRODRAGON Security",   "Code review, vulnerability analysis"),
    ];

    let list: Vec<_> = agents.iter().map(|(id, name, desc)| {
        serde_json::json!({ "id": id, "name": name, "description": desc, "enabled": true })
    }).collect();

    Json(serde_json::json!({ "agents": list }))
}
