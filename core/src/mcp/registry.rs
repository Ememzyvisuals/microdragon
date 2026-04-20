// Well-known MCP server registry
use std::collections::HashMap;

pub struct KnownMcpServer {
    pub name:        &'static str,
    pub description: &'static str,
    pub install_cmd: &'static str,
    pub env_required: &'static [&'static str],
}

pub const KNOWN_SERVERS: &[KnownMcpServer] = &[
    KnownMcpServer {
        name: "filesystem",
        description: "Read/write local files",
        install_cmd: "npx -y @modelcontextprotocol/server-filesystem",
        env_required: &[],
    },
    KnownMcpServer {
        name: "github",
        description: "GitHub repos, PRs, issues",
        install_cmd: "npx -y @modelcontextprotocol/server-github",
        env_required: &["GITHUB_TOKEN"],
    },
    KnownMcpServer {
        name: "fetch",
        description: "Web page fetching",
        install_cmd: "npx -y @modelcontextprotocol/server-fetch",
        env_required: &[],
    },
    KnownMcpServer {
        name: "postgres",
        description: "PostgreSQL database",
        install_cmd: "npx -y @modelcontextprotocol/server-postgres",
        env_required: &["POSTGRES_CONNECTION_STRING"],
    },
    KnownMcpServer {
        name: "slack",
        description: "Slack channels and messages",
        install_cmd: "npx -y @modelcontextprotocol/server-slack",
        env_required: &["SLACK_BOT_TOKEN"],
    },
    KnownMcpServer {
        name: "google-drive",
        description: "Google Drive files",
        install_cmd: "npx -y @modelcontextprotocol/server-gdrive",
        env_required: &["GOOGLE_SERVICE_ACCOUNT_KEY"],
    },
];
