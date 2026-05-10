// MCP HTTP/SSE transport — for remote servers
use anyhow::{Result, anyhow};
use serde_json::{json, Value};
use std::collections::HashMap;
use crate::mcp::{McpTool, McpResource};

pub async fn discover(url: &str, headers: &HashMap<String, String>)
    -> Result<(Vec<McpTool>, Vec<McpResource>)> {
    let client = reqwest::Client::new();
    let mut req = client.post(url)
        .json(&json!({
            "jsonrpc": "2.0", "id": 1,
            "method": "tools/list", "params": {}
        }));
    for (k, v) in headers {
        req = req.header(k, v);
    }
    
    let resp = req.send().await?.json::<Value>().await?;
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

pub async fn discover_ws(_url: &str) -> Result<(Vec<McpTool>, Vec<McpResource>)> {
    // WebSocket discovery — stub for now, treated as HTTP with ws→http URL rewrite
    Ok((vec![], vec![]))
}

pub async fn call_tool(url: &str, headers: &HashMap<String, String>,
                        tool_name: &str, args: Value) -> Result<Value> {
    let client = reqwest::Client::new();
    let mut req = client.post(url)
        .json(&json!({
            "jsonrpc": "2.0", "id": 1,
            "method": "tools/call",
            "params": {"name": tool_name, "arguments": args}
        }));
    for (k, v) in headers {
        req = req.header(k, v);
    }
    let resp = req.send().await?.json::<Value>().await?;
    if let Some(err) = resp["error"].as_object() {
        return Err(anyhow!("MCP HTTP error: {}", 
            err.get("message").and_then(|m| m.as_str()).unwrap_or("unknown")));
    }
    Ok(resp["result"].clone())
}

pub async fn call_tool_ws(url: &str, tool_name: &str, args: Value) -> Result<Value> {
    // Convert ws:// to http:// for simple cases
    let http_url = url.replacen("ws://", "http://", 1).replacen("wss://", "https://", 1);
    call_tool(&http_url, &HashMap::new(), tool_name, args).await
}
