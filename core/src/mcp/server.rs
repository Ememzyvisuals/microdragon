// Microdragon as an MCP server — expose its capabilities to other agents
// Other agents (Claude Code, OpenClaw, etc.) can call Microdragon as a tool server
use serde_json::{json, Value};
use anyhow::Result;

pub fn get_server_manifest() -> Value {
    json!({
        "name": "microdragon",
        "version": "0.1.1",
        "description": "🐉 Microdragon — full-spectrum AI agent",
        "tools": [
            {
                "name": "execute_task",
                "description": "Execute any task: code, security, research, design, gaming, trading",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "task": {"type": "string", "description": "Natural language task description"},
                        "agent": {"type": "string", "enum": ["coder","researcher","security","analyst","writer","automator","master"]}
                    },
                    "required": ["task"]
                }
            },
            {
                "name": "security_audit",
                "description": "Run OWASP SAST security audit on code",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "path": {"type": "string", "description": "File or directory path to audit"}
                    },
                    "required": ["path"]
                }
            }
        ]
    })
}
