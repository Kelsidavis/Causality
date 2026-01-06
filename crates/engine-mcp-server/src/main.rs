// Causality Engine - MCP Server
// Implements Model Context Protocol for Claude Code integration

mod protocol;
mod tools;

use anyhow::Result;
use protocol::{McpRequest, McpResponse};
use std::io::{self, BufRead, Write};
use tools::ToolRegistry;

fn main() -> Result<()> {
    env_logger::init();
    log::info!("Causality Engine MCP Server starting...");

    let mut tool_registry = ToolRegistry::new();
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    log::info!("MCP Server ready, listening on stdin...");

    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        log::debug!("Received: {}", line);

        // Parse JSON-RPC request
        let request: McpRequest = match serde_json::from_str(&line) {
            Ok(req) => req,
            Err(e) => {
                log::error!("Failed to parse request: {}", e);
                let error_response = McpResponse::error(
                    None,
                    -32700,
                    "Parse error".to_string(),
                );
                let response_json = serde_json::to_string(&error_response)?;
                writeln!(stdout, "{}", response_json)?;
                stdout.flush()?;
                continue;
            }
        };

        // Handle request
        let response = match request.method.as_str() {
            "initialize" => {
                log::info!("Initializing MCP server");
                McpResponse::success(
                    request.id,
                    serde_json::json!({
                        "protocolVersion": "2024-11-05",
                        "serverInfo": {
                            "name": "causality-engine",
                            "version": "0.1.0"
                        },
                        "capabilities": {
                            "tools": {}
                        }
                    }),
                )
            }
            "tools/list" => {
                log::info!("Listing available tools");
                let tools = tool_registry.list_tools();
                McpResponse::success(request.id, serde_json::json!({ "tools": tools }))
            }
            "tools/call" => {
                log::info!("Calling tool");
                match tool_registry.call_tool(&request.params) {
                    Ok(result) => McpResponse::success(request.id, result),
                    Err(e) => McpResponse::error(
                        request.id,
                        -32603,
                        format!("Tool execution failed: {}", e),
                    ),
                }
            }
            _ => McpResponse::error(
                request.id,
                -32601,
                format!("Method not found: {}", request.method),
            ),
        };

        // Send response
        let response_json = serde_json::to_string(&response)?;
        writeln!(stdout, "{}", response_json)?;
        stdout.flush()?;
    }

    log::info!("MCP Server shutting down");
    Ok(())
}
