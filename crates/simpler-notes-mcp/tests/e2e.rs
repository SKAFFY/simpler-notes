use std::io::{BufRead, BufReader, Read, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::Mutex;

use serde_json::{json, Value};
use tempfile::TempDir;

fn mcp_binary() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("../../target/debug/simpler-notes-mcp");
    p
}

struct McpClient {
    stdin: Mutex<ChildStdin>,
    stdout: BufReader<std::process::ChildStdout>,
    stderr: std::process::ChildStderr,
    _child: Child,
    next_id: u64,
}

impl McpClient {
    fn spawn(vault_path: &str) -> Self {
        let mut child = Command::new(mcp_binary())
            .env("VAULT_PATH", vault_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("failed to spawn MCP server");

        let stdin = child.stdin.take().expect("no stdin");
        let stdout = BufReader::new(child.stdout.take().expect("no stdout"));
        let stderr = child.stderr.take().expect("no stderr");

        McpClient {
            stdin: Mutex::new(stdin),
            stdout,
            stderr,
            _child: child,
            next_id: 1,
        }
    }

    fn read_stderr(&mut self) -> String {
        let mut buf = String::new();
        self.stderr.read_to_string(&mut buf).ok();
        buf
    }

    fn send_request(&mut self, method: &str, params: Option<Value>) -> Value {
        let id = self.next_id;
        self.next_id += 1;

        let request = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        });

        let body = serde_json::to_string(&request).unwrap();

        {
            let mut stdin = self.stdin.lock().unwrap();
            write!(stdin, "Content-Length: {}\r\n\r\n{}", body.len(), body).unwrap();
            stdin.flush().unwrap();
        }

        let mut line = String::new();
        loop {
            line.clear();
            let n = self.stdout.read_line(&mut line).unwrap_or(0);
            if n == 0 {
                let stderr = self.read_stderr();
                panic!("MCP server closed connection. stderr: {}", stderr);
            }
            let trimmed = line.trim();
            if trimmed.starts_with("Content-Length:") {
                let len_str = trimmed.trim_start_matches("Content-Length:").trim();
                let len: usize = len_str.parse().unwrap();

                let mut blank = String::new();
                self.stdout.read_line(&mut blank).unwrap();

                let mut body = vec![0u8; len];
                self.stdout.read_exact(&mut body).unwrap();
                return serde_json::from_slice(&body).unwrap();
            }
        }
    }
}

fn create_test_vault() -> (TempDir, PathBuf) {
    let dir = TempDir::new().unwrap();
    let vault_path = dir.path().to_path_buf();

    std::fs::create_dir(vault_path.join("notes")).unwrap();

    std::fs::write(
        vault_path.join("notes/alpha.md"),
        "[[beta]] [[gamma]] @tag1\n\n!01.01.2024",
    )
    .unwrap();

    std::fs::write(
        vault_path.join("notes/beta.md"),
        "A beta note.\n\n[[gamma|Gamma Link]]\n\n@tag2",
    )
    .unwrap();

    std::fs::write(vault_path.join("notes/gamma.md"), "Gamma note.\n\n[[delta]]").unwrap();

    // Non-md file in notes
    std::fs::write(vault_path.join("notes/alpha.txt"), "not a markdown note").unwrap();

    std::fs::create_dir_all(vault_path.join(".simpler-notes")).unwrap();

    (dir, vault_path)
}

#[test]
fn test_list_notes() {
    let (_dir, vault_path) = create_test_vault();
    let mut client = McpClient::spawn(vault_path.to_str().unwrap());

    let resp = client.send_request("list_notes", Some(json!({"path": "notes"})));
    let result = resp.get("result").and_then(|r| r.as_array());
    assert!(result.is_some(), "expected result array, got: {:?}", resp);
    let names: Vec<String> = result.unwrap().iter()
        .filter_map(|v| v.as_object())
        .filter(|o| o.get("type").and_then(|t| t.as_str()) == Some("file"))
        .filter_map(|o| o.get("name").and_then(|n| n.as_str().map(String::from)))
        .collect();
    assert!(names.contains(&"alpha.md".into()), "should contain alpha.md, got: {:?}", names);
    assert!(names.contains(&"beta.md".into()), "should contain beta.md");
    assert!(names.contains(&"gamma.md".into()), "should contain gamma.md");
}

#[test]
fn test_search_notes() {
    let (_dir, vault_path) = create_test_vault();
    let mut client = McpClient::spawn(vault_path.to_str().unwrap());

    let resp = client.send_request("search_notes", Some(json!({"query": "beta note"})));
    let result = resp.get("result").and_then(|r| r.as_array());
    assert!(result.is_some(), "expected array, got: {:?}", resp);
    let paths: Vec<&str> = result.unwrap().iter()
        .filter_map(|v| v.get("path").and_then(|p| p.as_str()))
        .collect();
    assert!(paths.iter().any(|p| p.contains("beta")), "beta should match 'beta note', paths: {:?}", paths);
}

#[test]
fn test_read_note() {
    let (_dir, vault_path) = create_test_vault();
    let mut client = McpClient::spawn(vault_path.to_str().unwrap());

    let resp = client.send_request("read_note", Some(json!({"path": "notes/alpha.md"})));
    let content = resp.get("result")
        .and_then(|r| r.get("content"))
        .and_then(|c| c.as_str());
    assert!(content.is_some(), "expected content in result, got: {:?}", resp);
    assert!(content.unwrap().contains("[[beta]]"));
}

#[test]
fn test_write_note_and_read_back() {
    let (_dir, vault_path) = create_test_vault();
    let mut client = McpClient::spawn(vault_path.to_str().unwrap());

    let resp = client.send_request(
        "write_note",
        Some(json!({
            "path": "notes/delta.md",
            "content": "## Delta\n\n[[alpha]] @newtag\n\n!15.06.2025"
        })),
    );
    assert!(resp.get("result").is_some(), "write failed: {:?}", resp);

    let resp = client.send_request("read_note", Some(json!({"path": "notes/delta.md"})));
    let content = resp.get("result")
        .and_then(|r| r.get("content"))
        .and_then(|c| c.as_str())
        .unwrap();
    assert!(content.contains("## Delta"));
    assert!(content.contains("[[alpha]]"));
}

#[test]
fn test_get_tags() {
    let (_dir, vault_path) = create_test_vault();
    let mut client = McpClient::spawn(vault_path.to_str().unwrap());

    let resp = client.send_request("get_tags", None);
    let result = resp.get("result").and_then(|r| r.as_array());
    assert!(result.is_some(), "expected array, got: {:?}", resp);
    let items: Vec<(String, u64)> = result.unwrap().iter()
        .filter_map(|v| {
            let tag = v.get("tag")?.as_str()?;
            let count = v.get("count")?.as_u64()?;
            Some((tag.to_string(), count))
        })
        .collect();
    assert!(items.contains(&("tag1".into(), 1)), "should contain tag1(1), got: {:?}", items);
    assert!(items.contains(&("tag2".into(), 1)), "should contain tag2(1), got: {:?}", items);
}

#[test]
fn test_get_diagnostics() {
    let (_dir, vault_path) = create_test_vault();
    let mut client = McpClient::spawn(vault_path.to_str().unwrap());

    let resp = client.send_request("get_diagnostics", None);
    assert!(resp.get("result").is_some(), "expected result, got: {:?}", resp);
}

#[test]
fn test_get_backlinks() {
    let (_dir, vault_path) = create_test_vault();
    let mut client = McpClient::spawn(vault_path.to_str().unwrap());

    // gamma is linked from alpha and beta
    let resp = client.send_request("get_backlinks", Some(json!({"path": "notes/gamma.md"})));
    let result = resp.get("result").and_then(|r| r.as_array());
    assert!(result.is_some(), "expected array, got: {:?}", resp);
    for item in result.unwrap() {
        let source = item.get("source").and_then(|s| s.as_str()).unwrap();
        let rel_source = source.rsplit('/').next().unwrap_or(source);
        let name = rel_source.trim_end_matches(".md");
        assert!(["alpha", "beta"].contains(&name), "unexpected source: {}", source);
    }
}

#[test]
fn test_get_outgoing_links() {
    let (_dir, vault_path) = create_test_vault();
    let mut client = McpClient::spawn(vault_path.to_str().unwrap());

    // alpha links to beta and gamma
    let resp = client.send_request("get_outgoing_links", Some(json!({"path": "notes/alpha.md"})));
    let result = resp.get("result").and_then(|r| r.as_array());
    assert!(result.is_some(), "expected array, got: {:?}", resp);
    let targets: Vec<&str> = result.unwrap().iter()
        .filter_map(|v| v.get("target").and_then(|s| s.as_str()))
        .collect();
    assert!(targets.contains(&"beta"), "alpha should link to beta, got: {:?}", targets);
    assert!(targets.contains(&"gamma"), "alpha should link to gamma, got: {:?}", targets);
}

#[test]
fn test_method_not_found() {
    let (_dir, vault_path) = create_test_vault();
    let mut client = McpClient::spawn(vault_path.to_str().unwrap());

    let resp = client.send_request("nonexistent_method", None);
    let err = resp.get("error").and_then(|e| e.get("code")).and_then(|c| c.as_i64());
    assert_eq!(err, Some(-32601), "expected method not found error");
}

#[test]
fn test_reindex() {
    let (_dir, vault_path) = create_test_vault();
    let mut client = McpClient::spawn(vault_path.to_str().unwrap());

    let resp = client.send_request("reindex", None);
    assert!(resp.get("result").is_some(), "reindex failed: {:?}", resp);
}

#[test]
fn test_get_dates() {
    let (_dir, vault_path) = create_test_vault();
    let mut client = McpClient::spawn(vault_path.to_str().unwrap());

    let resp = client.send_request("get_dates", None);
    let result = resp.get("result").and_then(|r| r.as_array());
    assert!(result.is_some(), "expected array, got: {:?}", resp);
    let dates: Vec<&str> = result.unwrap().iter()
        .filter_map(|v| v.get("date").and_then(|d| d.as_str()))
        .collect();
    assert!(dates.contains(&"2024-01-01"), "should contain 2024-01-01, got: {:?}", dates);
}

#[test]
fn test_get_dates_with_range() {
    let (_dir, vault_path) = create_test_vault();
    let mut client = McpClient::spawn(vault_path.to_str().unwrap());

    let resp = client.send_request("get_dates", Some(json!({"from": "01.01.2024", "to": "01.02.2024"})));
    let result = resp.get("result").and_then(|r| r.as_array());
    assert!(result.is_some(), "expected array, got: {:?}", resp);
    let dates: Vec<&str> = result.unwrap().iter()
        .filter_map(|v| v.get("date").and_then(|d| d.as_str()))
        .collect();
    assert!(dates.contains(&"2024-01-01"), "should contain 2024-01-01, got: {:?}", dates);
}

#[test]
fn test_get_dates_invalid_from_format() {
    let (_dir, vault_path) = create_test_vault();
    let mut client = McpClient::spawn(vault_path.to_str().unwrap());

    let resp = client.send_request("get_dates", Some(json!({"from": "invalid", "to": "01.01.2024"})));
    let error = resp.get("error").and_then(|e| e.get("message")).and_then(|m| m.as_str());
    assert!(error.is_some(), "expected error, got: {:?}", resp);
    assert!(error.unwrap().contains("Invalid date format for 'from'"));
}

#[test]
fn test_get_dates_invalid_to_format() {
    let (_dir, vault_path) = create_test_vault();
    let mut client = McpClient::spawn(vault_path.to_str().unwrap());

    let resp = client.send_request("get_dates", Some(json!({"from": "01.01.2024", "to": "bad"})));
    let error = resp.get("error").and_then(|e| e.get("message")).and_then(|m| m.as_str());
    assert!(error.is_some(), "expected error, got: {:?}", resp);
    assert!(error.unwrap().contains("Invalid date format for 'to'"));
}

#[test]
fn test_get_dates_from_only_returns_all() {
    let (_dir, vault_path) = create_test_vault();
    let mut client = McpClient::spawn(vault_path.to_str().unwrap());

    let resp = client.send_request("get_dates", Some(json!({"from": "01.01.2024"})));
    let result = resp.get("result").and_then(|r| r.as_array());
    assert!(result.is_some(), "expected array, got: {:?}", resp);
}

#[test]
fn test_validate_indexes() {
    let (_dir, vault_path) = create_test_vault();
    let mut client = McpClient::spawn(vault_path.to_str().unwrap());

    let resp = client.send_request("validate_indexes", None);
    let result = resp.get("result").and_then(|r| r.as_object());
    assert!(result.is_some(), "expected object, got: {:?}", resp);
    let notes = result.unwrap().get("total_notes").and_then(|n| n.as_u64()).unwrap_or(0);
    assert!(notes >= 1, "expected at least 1 note, got {}", notes);
}

#[test]
fn test_get_diagnostics_with_path() {
    let (_dir, vault_path) = create_test_vault();
    let mut client = McpClient::spawn(vault_path.to_str().unwrap());

    let resp = client.send_request("get_diagnostics", Some(json!({"path": "notes/alpha.md"})));
    let result = resp.get("result").and_then(|r| r.as_object());
    assert!(result.is_some(), "expected result object, got: {:?}", resp);
    let diags = result.unwrap().get("diagnostics").and_then(|d| d.as_array());
    assert!(diags.is_some(), "expected diagnostics array, got: {:?}", result);
}

#[test]
fn test_list_notes_with_subdir() {
    let dir = TempDir::new().unwrap();
    let vault_path = dir.path().to_path_buf();
    std::fs::create_dir(vault_path.join("notes")).unwrap();
    std::fs::create_dir(vault_path.join("notes/sub")).unwrap();
    std::fs::write(vault_path.join("notes/root.md"), "root").unwrap();
    std::fs::write(vault_path.join("notes/sub/deep.md"), "deep").unwrap();
    std::fs::create_dir_all(vault_path.join(".simpler-notes")).unwrap();

    let mut client = McpClient::spawn(vault_path.to_str().unwrap());

    let resp = client.send_request("list_notes", Some(json!({"path": "notes/sub"})));
    let result = resp.get("result").and_then(|r| r.as_array());
    assert!(result.is_some(), "expected array, got: {:?}", resp);
    let names: Vec<String> = result.unwrap().iter()
        .filter_map(|v| v.get("name").and_then(|n| n.as_str().map(String::from)))
        .collect();
    assert!(names.contains(&"deep.md".into()), "should list sub/deep.md, got: {:?}", names);
}

fn spawn_mcp_expecting_failure(vault_path: &str) -> std::process::Child {
    let child = Command::new(mcp_binary())
        .env("VAULT_PATH", vault_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn MCP server");
    child
}

#[test]
fn test_invalid_vault_path_exits_with_error() {
    let child = spawn_mcp_expecting_failure("/nonexistent/vault/path");
    let output = child.wait_with_output().expect("failed to wait");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Failed to open vault"), "expected error on stderr, got: {}", stderr);
}

#[test]
fn test_invalid_json_parse_error() {
    let (_dir, vault_path) = create_test_vault();
    let mut child = Command::new(mcp_binary())
        .env("VAULT_PATH", vault_path.to_str().unwrap())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn MCP server");
    use std::io::Write;
    let stdin = child.stdin.take().expect("no stdin");

    // Send garbage JSON (not valid JSON)
    let msg = b"Content-Length: 13\r\n\r\nnot valid json";
    let _ = (&stdin).write(msg);
    drop(stdin); // Close stdin so the server exits after processing

    let output = child.wait_with_output().expect("failed to wait");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Parse error"), "expected parse error on stderr, got: {}", stderr);
}

#[test]
fn test_resolve_link_finds_beta() {
    let (_dir, vault_path) = create_test_vault();
    let mut client = McpClient::spawn(vault_path.to_str().unwrap());

    let resp = client.send_request("resolve_link", Some(json!({"target": "beta"})));
    let path = resp.get("result")
        .and_then(|r| r.get("path"))
        .and_then(|p| p.as_str());
    assert!(path.is_some(), "expected path in result, got: {:?}", resp);
    assert!(
        path.unwrap().contains("beta.md"),
        "expected path to contain beta.md, got: {:?}",
        path
    );
}

#[test]
fn test_resolve_link_broken_returns_error() {
    let (_dir, vault_path) = create_test_vault();
    let mut client = McpClient::spawn(vault_path.to_str().unwrap());

    let resp = client.send_request("resolve_link", Some(json!({"target": "ghost"})));
    let error = resp.get("error")
        .and_then(|e| e.get("message"))
        .and_then(|m| m.as_str());
    assert!(error.is_some(), "expected error, got: {:?}", resp);
    assert!(error.unwrap().contains("Broken link"));
}

#[test]
fn test_resolve_link_ambiguous_returns_error() {
    let dir = TempDir::new().unwrap();
    let vault_path = dir.path().to_path_buf();

    std::fs::create_dir(vault_path.join("notes")).unwrap();
    std::fs::create_dir(vault_path.join("notes/sub")).unwrap();
    std::fs::write(vault_path.join("notes/duplicate.md"), "root dup").unwrap();
    std::fs::write(vault_path.join("notes/sub/duplicate.md"), "sub dup").unwrap();
    std::fs::create_dir_all(vault_path.join(".simpler-notes")).unwrap();

    let mut client = McpClient::spawn(vault_path.to_str().unwrap());

    let resp = client.send_request("resolve_link", Some(json!({"target": "duplicate"})));
    let error = resp.get("error")
        .and_then(|e| e.get("message"))
        .and_then(|m| m.as_str());
    assert!(error.is_some(), "expected error for ambiguous, got: {:?}", resp);
    assert!(error.unwrap().contains("Ambiguous link"));
}
