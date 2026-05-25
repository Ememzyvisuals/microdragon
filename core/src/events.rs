// microdragon-core/src/events.rs
// Agent event stream — pipeline emits these, TUI renders them in real time.
//
// This is what makes Microdragon look agentic: every tool call, every read,
// every reasoning step appears on screen as it happens.
//
// © 2026 EMEMZYVISUALS DIGITALS — Emmanuel Ariyo

use std::time::Instant;

/// A single event in the agent's work log
#[derive(Debug, Clone)]
pub struct AgentEvent {
    pub kind:      EventKind,
    pub text:      String,
    pub timestamp: std::time::Duration,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EventKind {
    /// Agent is thinking / narrating — plain white text
    Thought,
    /// Tool call — shown with `*` prefix, cyan
    ToolCall,
    /// File read — shown with `→` prefix, blue  
    FileRead,
    /// Web search — shown with `◉` prefix, yellow
    WebSearch,
    /// Memory recall — shown with `◈` prefix, purple
    MemoryRecall,
    /// Phase header — shown with phase number, green
    PhaseStart,
    /// Step completed — shown with `✓`, green
    Done,
    /// Warning — shown with `!`, orange
    Warning,
    /// Error — shown with `✗`, red
    Error,
    /// Final AI response text — rendered with markdown
    Response,
    /// Separator line
    Divider,
    /// User's input (echoed in the log)
    UserInput,
}

impl AgentEvent {
    pub fn new(kind: EventKind, text: impl Into<String>) -> Self {
        Self {
            kind,
            text: text.into(),
            timestamp: std::time::Duration::ZERO,
        }
    }

    pub fn with_time(mut self, start: &Instant) -> Self {
        self.timestamp = start.elapsed();
        self
    }

    /// Short prefix character for this event kind
    pub fn prefix(&self) -> &'static str {
        match self.kind {
            EventKind::Thought      => "  ",
            EventKind::ToolCall     => " *",
            EventKind::FileRead     => " →",
            EventKind::WebSearch    => " ◉",
            EventKind::MemoryRecall => " ◈",
            EventKind::PhaseStart   => " ▶",
            EventKind::Done         => " ✓",
            EventKind::Warning      => " !",
            EventKind::Error        => " ✗",
            EventKind::Response     => "  ",
            EventKind::Divider      => "  ",
            EventKind::UserInput    => "  ",
        }
    }
}

/// Channel alias for passing events from pipeline to TUI
pub type EventTx = tokio::sync::mpsc::UnboundedSender<AgentEvent>;
pub type EventRx = tokio::sync::mpsc::UnboundedReceiver<AgentEvent>;

pub fn channel() -> (EventTx, EventRx) {
    tokio::sync::mpsc::unbounded_channel()
}
