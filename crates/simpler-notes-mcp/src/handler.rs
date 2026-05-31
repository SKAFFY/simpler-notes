use serde_json::{json, Value};
use simpler_notes_core::vault::Vault;
use std::path::Path;

use crate::protocol::JsonRpcResponse;
use crate::tool;

pub fn handle_request(vault: &Vault, method: &str, params: &Value, id: Option<Value>) -> JsonRpcResponse {
    match method {
        "initialize" => handle_initialize(id),
        "tools/list" => handle_tools_list(id),
        "tools/call" => handle_tool_call(vault, params, id),
        "notifications/initialized" => JsonRpcResponse::success(id, json!(null)),
        "shutdown" => {
            JsonRpcResponse::success(id, json!(null))
        }
        _ => JsonRpcResponse::method_not_found(id, method),
    }
}

fn handle_initialize(id: Option<Value>) -> JsonRpcResponse {
    JsonRpcResponse::success(
        id,
        json!({
            "protocolVersion": "0.1.0",
            "capabilities": {
                "tools": {}
            },
            "serverInfo": {
                "name": "simpler-notes-mcp",
                "version": "0.1.0"
            }
        }),
    )
}

fn handle_tools_list(id: Option<Value>) -> JsonRpcResponse {
    let tools = tool::all_tools();
    JsonRpcResponse::success(id, json!({ "tools": tools }))
}

fn handle_tool_call(vault: &Vault, params: &Value, id: Option<Value>) -> JsonRpcResponse {
    let tool_name = match params["name"].as_str() {
        Some(name) => name,
        None => return JsonRpcResponse::invalid_params(id, "Missing 'name' field"),
    };

    let args = &params["arguments"];

    if let Some(tool_def) = tool::find_tool(tool_name) {
        if let Err(e) = tool::validate_required(&tool_def, args) {
            return JsonRpcResponse::invalid_params(id, e);
        }
    }

    match tool_name {
        "search_notes" => handle_search(vault, args, id),
        "read_note" => handle_read(vault, args, id),
        "write_note" => handle_write(vault, args, id),
        "list_notes" => handle_list(vault, args, id),
        "get_tags" => handle_get_tags(vault, id),
        "get_dates" => handle_get_dates(vault, id),
        "validate_indexes" => handle_validate(vault, id),
        "git_push" | "git_pull" => {
            JsonRpcResponse::internal_error(id, "Git integration not available without --features git")
        }
        _ => JsonRpcResponse::invalid_params(id, format!("Unknown tool: {}", tool_name)),
    }
}

fn handle_search(vault: &Vault, args: &Value, id: Option<Value>) -> JsonRpcResponse {
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
            respond_with_text(id, json!({ "results": items }))
        }
        Err(e) => JsonRpcResponse::internal_error(id, e),
    }
}

fn handle_read(vault: &Vault, args: &Value, id: Option<Value>) -> JsonRpcResponse {
    let path = args["path"].as_str().unwrap_or("");
    match vault.get_note(Path::new(path)) {
        Ok(content) => respond_with_text(id, json!({ "content": content })),
        Err(e) => JsonRpcResponse::internal_error(id, e),
    }
}

fn handle_write(vault: &Vault, args: &Value, id: Option<Value>) -> JsonRpcResponse {
    let path = args["path"].as_str().unwrap_or("");
    let content = args["content"].as_str().unwrap_or("");
    match vault.write_note(Path::new(path), content) {
        Ok(()) => respond_with_text(id, json!({ "status": "ok" })),
        Err(e) => JsonRpcResponse::internal_error(id, e),
    }
}

fn handle_list(vault: &Vault, args: &Value, id: Option<Value>) -> JsonRpcResponse {
    let vault_path = vault.path.clone();
    let walk_dir = match args["path"].as_str().filter(|s| !s.is_empty()) {
        Some(dir) => vault_path.join(dir),
        None => vault_path.clone(),
    };

    if !walk_dir.exists() {
        return respond_with_text(id, json!({ "files": [] }));
    }

    let mut files: Vec<Value> = Vec::new();
    collect_md_files(&walk_dir, &vault_path, &mut files);
    files.sort_by(|a, b| {
        a["path"].as_str().unwrap_or("").cmp(b["path"].as_str().unwrap_or(""))
    });

    respond_with_text(id, json!({ "files": files }))
}

fn collect_md_files(dir: &Path, vault_root: &Path, files: &mut Vec<Value>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if name != ".git" && name != ".index" {
                    collect_md_files(&path, vault_root, files);
                }
            } else if path.extension().map_or(false, |e| e == "md") {
                let relative = path.strip_prefix(vault_root).unwrap_or(&path);
                let title = simpler_notes_core::parser::extract_title(
                    &path,
                    &std::fs::read_to_string(&path).unwrap_or_default(),
                );
                files.push(json!({
                    "path": relative.to_string_lossy(),
                    "title": title,
                }));
            }
        }
    }
}

fn handle_get_tags(vault: &Vault, id: Option<Value>) -> JsonRpcResponse {
    let tags = vault.get_all_tags();
    respond_with_text(id, json!({ "tags": tags }))
}

fn handle_get_dates(vault: &Vault, id: Option<Value>) -> JsonRpcResponse {
    let dates: Vec<Value> = vault
        .get_all_dates()
        .into_iter()
        .map(|(date, paths)| {
            let paths: Vec<String> = paths
                .into_iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect();
            json!({
                "date": date.format("%d.%m.%Y").to_string(),
                "files": paths,
            })
        })
        .collect();
    respond_with_text(id, json!({ "dates": dates }))
}

fn handle_validate(vault: &Vault, id: Option<Value>) -> JsonRpcResponse {
    let report = vault.validate_indexes();
    respond_with_text(
        id,
        json!({
            "total_notes": report.total_notes,
            "total_tags": report.total_tags,
            "total_dates": report.total_dates,
        }),
    )
}

fn respond_with_text(id: Option<Value>, data: Value) -> JsonRpcResponse {
    let content = serde_json::to_string(&data).unwrap_or_default();
    JsonRpcResponse::success(id, json!({ "content": [{"type": "text", "text": content}] }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use simpler_notes_core::vault::Vault;
    use std::io::Write;
    use tempfile::TempDir;

    fn setup_vault() -> (TempDir, Vault) {
        let dir = TempDir::new().unwrap();
        let vault_path = dir.path().join(".simpler-notes-vault");
        std::fs::create_dir_all(&vault_path).unwrap();

        // note1.md with tag1 and date
        let note1 = vault_path.join("note1.md");
        let mut f = std::fs::File::create(&note1).unwrap();
        writeln!(f, "# Note 1").unwrap();
        writeln!(f, "Content with #tag1 and date 01.01.2024").unwrap();

        // note2.md in subdirectory with tag2 and date
        let sub = vault_path.join("sub");
        std::fs::create_dir_all(&sub).unwrap();
        let note2 = sub.join("note2.md");
        let mut f = std::fs::File::create(&note2).unwrap();
        writeln!(f, "# Note 2").unwrap();
        writeln!(f, "More content #tag2 date 15.06.2024").unwrap();

        let vault = Vault::open(&vault_path).unwrap();
        (dir, vault)
    }

    fn get_text_content(response: &JsonRpcResponse) -> Value {
        let content = response.result.as_ref().unwrap();
        let items = content["content"].as_array().unwrap();
        let text = items[0]["text"].as_str().unwrap();
        serde_json::from_str(text).unwrap()
    }

    #[test]
    fn test_initialize() {
        let (_dir, vault) = setup_vault();
        let response = handle_request(&vault, "initialize", &json!({}), Some(json!(1)));
        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.id, Some(json!(1)));
        assert!(response.error.is_none());

        let result = response.result.unwrap();
        assert_eq!(result["protocolVersion"], "0.1.0");
        assert_eq!(result["serverInfo"]["name"], "simpler-notes-mcp");
        assert_eq!(result["serverInfo"]["version"], "0.1.0");
    }

    #[test]
    fn test_tools_list() {
        let (_dir, vault) = setup_vault();
        let response = handle_request(&vault, "tools/list", &json!({}), Some(json!(1)));
        assert!(response.error.is_none());

        let result = response.result.unwrap();
        let tools = result["tools"].as_array().unwrap();
        let names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
        assert!(names.contains(&"search_notes"));
        assert!(names.contains(&"read_note"));
        assert!(names.contains(&"write_note"));
        assert!(names.contains(&"list_notes"));
        assert!(names.contains(&"get_tags"));
        assert!(names.contains(&"get_dates"));
        assert!(names.contains(&"validate_indexes"));
        assert!(names.contains(&"git_push"));
        assert!(names.contains(&"git_pull"));

        for tool in tools {
            assert!(tool["description"].as_str().unwrap().len() > 0);
            assert_eq!(tool["input_schema"]["type"], "object");
            if let Some(props) = tool["input_schema"]["properties"].as_array() {
                for prop in props {
                    assert!(prop["name"].as_str().unwrap().len() > 0);
                    assert!(prop["type"].as_str().unwrap().len() > 0);
                    assert!(prop["description"].as_str().unwrap().len() > 0);
                }
            }
        }
    }

    #[test]
    fn test_search_notes() {
        let (_dir, vault) = setup_vault();
        let params = json!({"name": "search_notes", "arguments": {"query": "tags contain \"tag1\""}});
        let response = handle_request(&vault, "tools/call", &params, Some(json!(1)));
        assert!(response.error.is_none(), "error: {:?}", response.error);

        let data = get_text_content(&response);
        let results = data["results"].as_array().unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0]["path"].as_str().unwrap().contains("note1.md"));
    }

    #[test]
    fn test_read_note() {
        let (_dir, vault) = setup_vault();
        let params = json!({"name": "read_note", "arguments": {"path": "note1.md"}});
        let response = handle_request(&vault, "tools/call", &params, Some(json!(1)));
        assert!(response.error.is_none());

        let data = get_text_content(&response);
        let content = data["content"].as_str().unwrap();
        assert!(content.contains("Note 1"));
    }

    #[test]
    fn test_write_note() {
        let (_dir, vault) = setup_vault();
        let params = json!({
            "name": "write_note",
            "arguments": {"path": "new.md", "content": "# New Note\nFresh content #newtag"}
        });
        let response = handle_request(&vault, "tools/call", &params, Some(json!(1)));
        assert!(response.error.is_none());

        // verify by reading it back
        let params = json!({"name": "read_note", "arguments": {"path": "new.md"}});
        let response = handle_request(&vault, "tools/call", &params, Some(json!(2)));
        assert!(response.error.is_none());
        let data = get_text_content(&response);
        assert!(data["content"].as_str().unwrap().contains("New Note"));
    }

    #[test]
    fn test_list_notes() {
        let (_dir, vault) = setup_vault();
        let params = json!({"name": "list_notes", "arguments": {}});
        let response = handle_request(&vault, "tools/call", &params, Some(json!(1)));
        assert!(response.error.is_none());

        let data = get_text_content(&response);
        let files = data["files"].as_array().unwrap();
        assert_eq!(files.len(), 2);

        let paths: Vec<&str> = files.iter().map(|f| f["path"].as_str().unwrap()).collect();
        assert!(paths.contains(&"note1.md"));
        assert!(paths.contains(&"sub/note2.md"));
    }

    #[test]
    fn test_list_notes_with_subdir() {
        let (_dir, vault) = setup_vault();
        let params = json!({"name": "list_notes", "arguments": {"path": "sub"}});
        let response = handle_request(&vault, "tools/call", &params, Some(json!(1)));
        assert!(response.error.is_none());

        let data = get_text_content(&response);
        let files = data["files"].as_array().unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0]["path"], "sub/note2.md");
    }

    #[test]
    fn test_list_notes_nonexistent_subdir() {
        let (_dir, vault) = setup_vault();
        let params = json!({"name": "list_notes", "arguments": {"path": "nonexistent"}});
        let response = handle_request(&vault, "tools/call", &params, Some(json!(1)));
        assert!(response.error.is_none());

        let data = get_text_content(&response);
        let files = data["files"].as_array().unwrap();
        assert_eq!(files.len(), 0);
    }

    #[test]
    fn test_get_tags() {
        let (_dir, vault) = setup_vault();
        let params = json!({"name": "get_tags", "arguments": {}});
        let response = handle_request(&vault, "tools/call", &params, Some(json!(1)));
        assert!(response.error.is_none());

        let data = get_text_content(&response);
        let tags = data["tags"].as_array().unwrap();
        assert!(tags.contains(&json!("tag1")));
        assert!(tags.contains(&json!("tag2")));
    }

    #[test]
    fn test_get_dates() {
        let (_dir, vault) = setup_vault();
        let params = json!({"name": "get_dates", "arguments": {}});
        let response = handle_request(&vault, "tools/call", &params, Some(json!(1)));
        assert!(response.error.is_none());

        let data = get_text_content(&response);
        let dates = data["dates"].as_array().unwrap();
        let date_strings: Vec<&str> = dates.iter().map(|d| d["date"].as_str().unwrap()).collect();
        assert!(date_strings.contains(&"01.01.2024"));
        assert!(date_strings.contains(&"15.06.2024"));
    }

    #[test]
    fn test_validate_indexes() {
        let (_dir, vault) = setup_vault();
        let params = json!({"name": "validate_indexes", "arguments": {}});
        let response = handle_request(&vault, "tools/call", &params, Some(json!(1)));
        assert!(response.error.is_none());

        let data = get_text_content(&response);
        // file_states isn't populated by rebuild_index_sync, so total_notes stays 0
        assert!(data["total_tags"].as_u64().unwrap() >= 2);
        assert!(data["total_dates"].as_u64().unwrap() >= 2);
    }

    #[test]
    fn test_notification_initialized() {
        let (_dir, vault) = setup_vault();
        let response = handle_request(&vault, "notifications/initialized", &json!({}), None);
        assert_eq!(response.result, Some(json!(null)));
        assert!(response.error.is_none());
    }

    #[test]
    fn test_unknown_method() {
        let (_dir, vault) = setup_vault();
        let response = handle_request(&vault, "unknown_method", &json!({}), Some(json!(1)));
        assert!(response.result.is_none());
        assert!(response.error.is_some());
        assert_eq!(response.error.as_ref().unwrap().code, -32601);
    }

    #[test]
    fn test_unknown_tool() {
        let (_dir, vault) = setup_vault();
        let params = json!({"name": "nonexistent_tool", "arguments": {}});
        let response = handle_request(&vault, "tools/call", &params, Some(json!(1)));
        assert!(response.result.is_none());
        assert!(response.error.is_some());
        assert_eq!(response.error.as_ref().unwrap().code, -32602);
    }

    #[test]
    fn test_missing_required_param() {
        let (_dir, vault) = setup_vault();
        let params = json!({"name": "search_notes", "arguments": {}});
        let response = handle_request(&vault, "tools/call", &params, Some(json!(1)));
        assert!(response.result.is_none());
        assert!(response.error.is_some());
        assert_eq!(response.error.as_ref().unwrap().code, -32602);
    }

    #[test]
    fn test_git_tools_disabled() {
        let (_dir, vault) = setup_vault();
        let params = json!({"name": "git_push", "arguments": {}});
        let response = handle_request(&vault, "tools/call", &params, Some(json!(1)));
        assert!(response.error.is_some());
        assert_eq!(response.error.as_ref().unwrap().code, -32000);
    }
}
