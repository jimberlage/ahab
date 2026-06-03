use serde_json::{json, Value};
use std::io::{self, BufRead, Write};

use crate::error::Result;

/// Launch the MCP server using STDIO transport with simple JSON-RPC
pub async fn mcp() -> Result<()> {
    tracing::debug!("Starting MCP server");
    
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    
    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        
        tracing::debug!("Received: {}", line);
        
        let request: Value = serde_json::from_str(&line)?;
        let response = handle_request(request).await;
        
        let response_json = serde_json::to_string(&response)?;
        writeln!(stdout, "{}", response_json)?;
        stdout.flush()?;
        
        tracing::debug!("Sent: {}", response_json);
    }
    
    Ok(())
}

async fn handle_request(request: Value) -> Value {
    let method = request.get("method").and_then(|v| v.as_str());
    let id = request.get("id").cloned();
    
    match method {
        Some("initialize") => {
            json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "protocolVersion": "2024-11-05",
                    "capabilities": {
                        "tools": {}
                    },
                    "serverInfo": {
                        "name": "ahab",
                        "version": env!("CARGO_PKG_VERSION")
                    }
                }
            })
        }
        Some("tools/list") => {
            json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "tools": [
                        {
                            "name": "convert",
                            "description": "Convert Aha pages to markdown in a session",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "pages": {
                                        "type": "array",
                                        "items": { "type": "string" },
                                        "description": "Page URLs or slugs"
                                    },
                                    "profile": {
                                        "type": "string",
                                        "description": "Profile to use (optional)"
                                    },
                                    "session": {
                                        "type": "string",
                                        "description": "Session ID (optional)"
                                    }
                                },
                                "required": ["pages"]
                            }
                        },
                        {
                            "name": "push",
                            "description": "Push session epics to Aha",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "session": {
                                        "type": "string",
                                        "description": "Session ID to push"
                                    },
                                    "profile": {
                                        "type": "string",
                                        "description": "Profile to use (optional)"
                                    }
                                },
                                "required": ["session"]
                            }
                        },
                        {
                            "name": "list_sessions",
                            "description": "List all available sessions",
                            "inputSchema": {
                                "type": "object",
                                "properties": {}
                            }
                        },
                        {
                            "name": "delete_session",
                            "description": "Delete a session",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "session": {
                                        "type": "string",
                                        "description": "Session ID to delete"
                                    }
                                },
                                "required": ["session"]
                            }
                        }
                    ]
                }
            })
        }
        Some("tools/call") => {
            let params = request.get("params");
            handle_tool_call(id, params).await
        }
        _ => {
            json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {
                    "code": -32601,
                    "message": "Method not found"
                }
            })
        }
    }
}

async fn handle_tool_call(id: Option<Value>, params: Option<&Value>) -> Value {
    let tool_name = params
        .and_then(|p| p.get("name"))
        .and_then(|v| v.as_str());
    
    let arguments = params.and_then(|p| p.get("arguments"));
    
    match tool_name {
        Some("convert") => {
            let pages: Vec<String> = arguments
                .and_then(|a| a.get("pages"))
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_default();
            
            let profile: Option<String> = arguments
                .and_then(|a| a.get("profile"))
                .and_then(|v| serde_json::from_value(v.clone()).ok());
            
            let session: Option<String> = arguments
                .and_then(|a| a.get("session"))
                .and_then(|v| serde_json::from_value(v.clone()).ok());
            
            match crate::cli::convert(pages, profile, session).await {
                Ok(_) => json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "content": [{
                            "type": "text",
                            "text": "Pages successfully converted to markdown"
                        }]
                    }
                }),
                Err(e) => json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": {
                        "code": -32000,
                        "message": e.to_string()
                    }
                })
            }
        }
        Some("push") => {
            let session: String = arguments
                .and_then(|a| a.get("session"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            
            let profile: Option<String> = arguments
                .and_then(|a| a.get("profile"))
                .and_then(|v| serde_json::from_value(v.clone()).ok());
            
            match crate::cli::push(session, profile).await {
                Ok(_) => json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "content": [{
                            "type": "text",
                            "text": "Epics successfully pushed to Aha"
                        }]
                    }
                }),
                Err(e) => json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": {
                        "code": -32000,
                        "message": e.to_string()
                    }
                })
            }
        }
        Some("list_sessions") => {
            match list_sessions_impl().await {
                Ok(text) => json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "content": [{
                            "type": "text",
                            "text": text
                        }]
                    }
                }),
                Err(e) => json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": {
                        "code": -32000,
                        "message": e.to_string()
                    }
                })
            }
        }
        Some("delete_session") => {
            let session: String = arguments
                .and_then(|a| a.get("session"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            
            match crate::cli::delete_session(session.clone()).await {
                Ok(_) => json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "content": [{
                            "type": "text",
                            "text": format!("Session {} successfully deleted", session)
                        }]
                    }
                }),
                Err(e) => json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": {
                        "code": -32000,
                        "message": e.to_string()
                    }
                })
            }
        }
        _ => {
            json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {
                    "code": -32602,
                    "message": "Unknown tool"
                }
            })
        }
    }
}

async fn list_sessions_impl() -> Result<String> {
    use crate::config::ConfigManager;
    use crate::session::Session;
    
    let config_manager = ConfigManager::new()?;
    let sessions_dir = config_manager.sessions_dir();
    let sessions = Session::list_all(&sessions_dir)?;
    
    if sessions.is_empty() {
        return Ok("No sessions found".to_string());
    }
    
    let mut output = String::from("Available sessions:\n\n");
    for session in sessions {
        output.push_str(&format!("Session ID: {}\n", session.session_id));
        output.push_str(&format!("  Created: {}\n", session.created_at));
        output.push_str(&format!("  Profile: {}\n", session.profile));
        if let Some(page_name) = &session.page_name {
            output.push_str(&format!("  Page: {}\n", page_name));
        }
        output.push_str(&format!("  Pages: {}\n", session.page_manifest.len()));
        output.push_str(&format!("  Epics: {}\n", session.epic_manifest.len()));
        output.push_str("\n");
    }
    
    Ok(output)
}
