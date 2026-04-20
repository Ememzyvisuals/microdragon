// microdragon/core/src/mcp/mod.rs
//
// MCP — Model Context Protocol support
//
// The official Rust SDK for MCP (RMCP) provides a type-safe, async implementation.
// We implement MCP JSON-RPC 2.0 directly for maximum control and zero extra deps.
//
// Microdragon can:
//   - Connect to any MCP server (stdio or HTTP/SSE)
//   - Discover available tools, resources, prompts
//   - Execute tools and return results to the AI
//   - Expose itself as an MCP server for other agents
//
// © 2026 EMEMZYVISUALS DIGITALS — Emmanuel Ariyo

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub mod client;
pub mod server;
pub mod registry;

/// MCP Tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name:        String,
    pub description: String,
    pub input_schema: Value,
}

/// MCP Resource definition  
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResource {
    pub uri:         String,
    pub name:        String,
    pub description: Option<String>,
    pub mime_type:   Option<String>,
}

/// MCP Server connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub name:       String,
    pub transport:  McpTransport,
    pub enabled:    bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum McpTransport {
    /// Local process via stdin/stdout
    Stdio { command: String, args: Vec<String>, env: HashMap<String, String> },
    /// Remote server via HTTP/SSE
    Http { url: String, headers: HashMap<String, String> },
    /// Remote server via WebSocket
    WebSocket { url: String },
}

/// Registered MCP servers with their discovered capabilities
#[derive(Default)]
pub struct McpRegistry {
    servers: Arc<RwLock<HashMap<String, RegisteredServer>>>,
}

struct RegisteredServer {
    config:    McpServerConfig,
    tools:     Vec<McpTool>,
    resources: Vec<McpResource>,
    connected: bool,
}

impl McpRegistry {
    pub fn new() -> Self { Self::default() }

    /// Register and connect to an MCP server
    pub async fn register(&self, config: McpServerConfig) -> Result<()> {
        let name = config.name.clone();
        let (tools, resources) = self.discover(&config).await?;
        
        let mut servers = self.servers.write().await;
        servers.insert(name.clone(), RegisteredServer {
            config, tools, resources, connected: true,
        });
        
        tracing::info!("MCP server '{}' registered", name);
        Ok(())
    }

    /// Discover tools and resources from a server
    async fn discover(&self, config: &McpServerConfig) -> Result<(Vec<McpTool>, Vec<McpResource>)> {
        match &config.transport {
            McpTransport::Stdio { command, args, env } => {
                client::stdio::discover(command, args, env).await
            }
            McpTransport::Http { url, headers } => {
                client::http::discover(url, headers).await
            }
            McpTransport::WebSocket { url } => {
                // WebSocket discovery — same JSON-RPC protocol
                client::http::discover_ws(url).await
            }
        }
    }

    /// Execute a tool on a registered server
    pub async fn call_tool(&self, server_name: &str, tool_name: &str, args: Value) -> Result<Value> {
        let servers = self.servers.read().await;
        let server = servers.get(server_name)
            .ok_or_else(|| anyhow!("MCP server '{}' not registered", server_name))?;
        
        if !server.connected {
            return Err(anyhow!("MCP server '{}' is not connected", server_name));
        }

        match &server.config.transport {
            McpTransport::Stdio { command, args: cmd_args, env } => {
                client::stdio::call_tool(command, cmd_args, env, tool_name, args).await
            }
            McpTransport::Http { url, headers } => {
                client::http::call_tool(url, headers, tool_name, args).await
            }
            McpTransport::WebSocket { url } => {
                client::http::call_tool_ws(url, tool_name, args).await
            }
        }
    }

    /// Get all tools across all registered servers
    pub async fn all_tools(&self) -> Vec<(String, McpTool)> {
        let servers = self.servers.read().await;
        let mut tools = Vec::new();
        for (server_name, server) in servers.iter() {
            if server.connected {
                for tool in &server.tools {
                    tools.push((server_name.clone(), tool.clone()));
                }
            }
        }
        tools
    }

    /// List all registered servers and their status
    pub async fn list(&self) -> Vec<(String, bool, usize)> {
        let servers = self.servers.read().await;
        servers.iter()
            .map(|(name, s)| (name.clone(), s.connected, s.tools.len()))
            .collect()
    }
}

/// Built-in MCP servers Microdragon ships with
pub fn builtin_servers() -> Vec<McpServerConfig> {
    vec![
        // Filesystem access (read/write local files)
        McpServerConfig {
            name: "filesystem".to_string(),
            transport: McpTransport::Stdio {
                command: "npx".to_string(),
                args: vec!["-y".to_string(), "@modelcontextprotocol/server-filesystem".to_string(), ".".to_string()],
                env: HashMap::new(),
            },
            enabled: true,
        },
        // GitHub integration
        McpServerConfig {
            name: "github".to_string(),
            transport: McpTransport::Stdio {
                command: "npx".to_string(),
                args: vec!["-y".to_string(), "@modelcontextprotocol/server-github".to_string()],
                env: {
                    let mut m = HashMap::new();
                    m.insert("GITHUB_PERSONAL_ACCESS_TOKEN".to_string(), 
                             std::env::var("GITHUB_TOKEN").unwrap_or_default());
                    m
                },
            },
            enabled: std::env::var("GITHUB_TOKEN").is_ok(),
        },
        // Web search / fetch
        McpServerConfig {
            name: "fetch".to_string(),
            transport: McpTransport::Stdio {
                command: "npx".to_string(),
                args: vec!["-y".to_string(), "@modelcontextprotocol/server-fetch".to_string()],
                env: HashMap::new(),
            },
            enabled: true,
        },
    ]
}
