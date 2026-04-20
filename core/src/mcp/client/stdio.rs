// MCP stdio transport — for local server processes
use anyhow::{Result, anyhow};
use serde_json::{json, Value};
use std::collections::HashMap;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use crate::mcp::{McpTool, McpResource};

/// Send a JSON-RPC request to an MCP stdio server and get response
async fn jsonrpc_request(command: &str, args: &[String], env: &HashMap<String, String>,
                          method: &str, params: Value) -> Result<Value> {
    let mut child = Command::new(command)
        .args(args)
        .envs(env)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()?;

    let mut stdin = child.stdin.take().ok_or_else(|| anyhow!("No stdin"))?;
    let stdout = child.stdout.take().ok_or_else(|| anyhow!("No stdout"))?;

    // Send initialize first (MCP handshake)
    let init_msg = json!({
        "jsonrpc": "2.0", "id": 1, "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {"tools": {}, "resources": {}},
            "clientInfo": {"name": "microdragon", "version": "0.1.1"}
        }
    });
    let init_str = format!("{}\n", serde_json::to_string(&init_msg)?);
    stdin.write_all(init_str.as_bytes()).await?;

    // Send the actual request
    let req = json!({"jsonrpc": "2.0", "id": 2, "method": method, "params": params});
    let req_str = format!("{}\n", serde_json::to_string(&req)?);
    stdin.write_all(req_str.as_bytes()).await?;
    drop(stdin);

    // Read responses
    let mut reader = BufReader::new(stdout);
    let mut response_value = Value::Null;
    let mut line = String::new();
    let mut read_count = 0;

    while reader.read_line(&mut line).await? > 0 && read_count < 10 {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            if let Ok(val) = serde_json::from_str::<Value>(trimmed) {
                if val.get("id") == Some(&json!(2)) {
                    response_value = val;
                    break;
                }
            }
        }
        line.clear();
        read_count += 1;
    }

    let _ = child.kill().await;
    Ok(response_value)
}

pub async fn discover(command: &str, args: &[String], env: &HashMap<String, String>)
    -> Result<(Vec<McpTool>, Vec<McpResource>)> {
    // List tools
    let resp = jsonrpc_request(command, args, env, "tools/list", json!({})).await
        .unwrap_or(Value::Null);
    
    let tools = resp["result"]["tools"].as_array()
        .map(|arr| arr.iter().filter_map(|t| {
            Some(McpTool {
                name: t["name"].as_str()?.to_string(),
                description: t["description"].as_str().unwrap_or("").to_string(),
                input_schema: t["inputSchema"].clone(),
            })
        }).collect())
        .unwrap_or_default();

    Ok((tools, vec![]))
}

pub async fn call_tool(command: &str, args: &[String], env: &HashMap<String, String>,
                        tool_name: &str, tool_args: Value) -> Result<Value> {
    let resp = jsonrpc_request(
        command, args, env,
        "tools/call",
        json!({"name": tool_name, "arguments": tool_args})
    ).await?;
    
    if let Some(err) = resp["error"].as_object() {
        return Err(anyhow!("MCP tool error: {}", err.get("message")
            .and_then(|m| m.as_str()).unwrap_or("unknown")));
    }
    
    Ok(resp["result"].clone())
}
