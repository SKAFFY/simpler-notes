use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use simpler_notes_core::vault::Vault;
use std::io::{self, BufRead, Write};
use std::path::Path;

#[derive(Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Value,
}

#[derive(Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
}

fn send_response(id: Option<Value>, result: Option<Value>, error: Option<JsonRpcError>) {
    let response = JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result,
        error,
    };
    let serialized = serde_json::to_string(&response).unwrap();
    println!("{}", serialized);
    io::stdout().flush().ok();
}

fn handle_request(vault: &Vault, method: &str, params: &Value, id: Option<Value>) {
    match method {
        "initialize" => {
            send_response(
                id,
                Some(json!({
                    "protocolVersion": "0.1.0",
                    "capabilities": {
                        "tools": {}
                    },
                    "serverInfo": {
                        "name": "simpler-notes-mcp",
                        "version": "0.1.0"
                    }
                })),
                None,
            );
        }
        "tools/list" => {
            send_response(
                id,
                Some(json!({
                    "tools": [
                        {
                            "name": "search_notes",
                            "description": "Search notes using query language. Supports: tags contain \"tag\", date before DD.MM.YYYY, date after DD.MM.YYYY, and/or combinators, plain text search.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "query": {"type": "string", "description": "Search query"}
                                },
                                "required": ["query"]
                            }
                        },
                        {
                            "name": "read_note",
                            "description": "Read the content of a note file.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "path": {"type": "string", "description": "Relative path to the note file"}
                                },
                                "required": ["path"]
                            }
                        },
                        {
                            "name": "write_note",
                            "description": "Create or update a note file.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "path": {"type": "string", "description": "Relative path to the note file"},
                                    "content": {"type": "string", "description": "Markdown content"}
                                },
                                "required": ["path", "content"]
                            }
                        },
                        {
                            "name": "list_notes",
                            "description": "List all markdown notes in the vault.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "path": {"type": "string", "description": "Optional subdirectory path"}
                                },
                                "required": []
                            }
                        },
                        {
                            "name": "get_tags",
                            "description": "Get all tags used in the vault.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {},
                                "required": []
                            }
                        },
                        {
                            "name": "get_dates",
                            "description": "Get all dates found in notes.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {},
                                "required": []
                            }
                        },
                        {
                            "name": "validate_indexes",
                            "description": "Validate index integrity.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {},
                                "required": []
                            }
                        },
                        {
                            "name": "git_push",
                            "description": "Push local commits to remote git repository.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {},
                                "required": []
                            }
                        },
                        {
                            "name": "git_pull",
                            "description": "Pull latest changes from remote git repository.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {},
                                "required": []
                            }
                        }
                    ]
                })),
                None,
            );
        }
        "tools/call" => {
            let tool_name = params["name"].as_str().unwrap_or("");
            let tool_args = &params["arguments"];
            handle_tool_call(vault, tool_name, tool_args, id);
        }
        "notifications/initialized" => {}
        "shutdown" => {
            send_response(id, Some(json!(null)), None);
            std::process::exit(0);
        }
        _ => {
            send_response(
                id,
                None,
                Some(JsonRpcError {
                    code: -32601,
                    message: format!("Method not found: {}", method),
                }),
            );
        }
    }
}

fn handle_tool_call(vault: &Vault, tool: &str, args: &Value, id: Option<Value>) {
    let result = match tool {
        "search_notes" => {
            let query = args["query"].as_str().unwrap_or("");
            match vault.search(query) {
                Ok(results) => {
                    let items: Vec<Value> = results
                        .into_iter()
                        .map(|r| {
                            json!({
                                "path": r.path.to_string_lossy(),
                                "title": r.title,
                            })
                        })
                        .collect();
                    Ok(json!({ "results": items }))
                }
                Err(e) => Err(e),
            }
        }
        "read_note" => {
            let path = args["path"].as_str().unwrap_or("");
            match vault.get_note(Path::new(path)) {
                Ok(content) => Ok(json!({ "content": content })),
                Err(e) => Err(e),
            }
        }
        "write_note" => {
            let path = args["path"].as_str().unwrap_or("");
            let content = args["content"].as_str().unwrap_or("");
            match vault.write_note(Path::new(path), content) {
                Ok(()) => Ok(json!({ "status": "ok" })),
                Err(e) => Err(e),
            }
        }
        "list_notes" => {
            let tags = vault.get_all_tags();
            Ok(json!({ "tags": tags }))
        }
        "get_tags" => {
            let tags = vault.get_all_tags();
            Ok(json!({ "tags": tags }))
        }
        "get_dates" => {
            let dates: Vec<Value> = vault
                .get_all_dates()
                .into_iter()
                .map(|(date, paths)| {
                    let paths: Vec<String> =
                        paths.into_iter().map(|p| p.to_string_lossy().to_string()).collect();
                    json!({ "date": date.format("%d.%m.%Y").to_string(), "files": paths })
                })
                .collect();
            Ok(json!({ "dates": dates }))
        }
        "validate_indexes" => {
            let report = vault.validate_indexes();
            Ok(json!({
                "total_notes": report.total_notes,
                "total_tags": report.total_tags,
                "total_dates": report.total_dates,
            }))
        }
        "git_push" | "git_pull" => {
            Err("Git integration not available without --features git".to_string())
        }
        _ => {
            send_response(
                id,
                None,
                Some(JsonRpcError {
                    code: -32602,
                    message: format!("Unknown tool: {}", tool),
                }),
            );
            return;
        }
    };

    match result {
        Ok(data) => send_response(id, Some(json!({ "content": [{"type": "text", "text": serde_json::to_string(&data).unwrap_or_default()}] })), None),
        Err(e) => send_response(id, None, Some(JsonRpcError { code: -32000, message: e })),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let vault_path = if let Some(pos) = args.iter().position(|a| a == "--vault") {
        args.get(pos + 1).cloned().unwrap_or_else(|| {
            eprintln!("Usage: simpler-notes-mcp --vault <path>");
            std::process::exit(1);
        })
    } else {
        eprintln!("Usage: simpler-notes-mcp --vault <path>");
        std::process::exit(1);
    };

    let vault = match Vault::open(Path::new(&vault_path)) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Failed to open vault: {}", e);
            std::process::exit(1);
        }
    };

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        match line {
            Ok(line) => {
                let trimmed = line.trim().to_string();
                if trimmed.is_empty() {
                    continue;
                }
                match serde_json::from_str::<JsonRpcRequest>(&trimmed) {
                    Ok(request) => {
                        handle_request(&vault, &request.method, &request.params, request.id);
                    }
                    Err(e) => {
                        send_response(
                            None,
                            None,
                            Some(JsonRpcError {
                                code: -32700,
                                message: format!("Parse error: {}", e),
                            }),
                        );
                    }
                }
            }
            Err(e) => {
                eprintln!("Error reading stdin: {}", e);
                break;
            }
        }
    }
}
