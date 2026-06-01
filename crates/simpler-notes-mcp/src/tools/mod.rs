use std::sync::Arc;
use serde_json::json;
use simpler_notes_core::vault::Vault;
use crate::dispatcher::{Dispatcher, ToolInfo};

pub mod search_notes;
pub mod read_note;
pub mod write_note;
pub mod list_notes;
pub mod get_tags;
pub mod get_dates;
pub mod get_backlinks;
pub mod get_outgoing_links;
pub mod resolve_link;
#[cfg(feature = "git")]
pub mod git_push;
#[cfg(feature = "git")]
pub mod git_pull;
pub mod validate_indexes;
pub mod reindex;
pub mod get_diagnostics;

pub fn register_all(dispatcher: &mut Dispatcher, vault: Arc<Vault>) {
    dispatcher.register("search_notes", Arc::new(search_notes::SearchNotesTool::new(vault.clone())), ToolInfo {
        description: "Search notes in the vault by full-text query".into(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "query": {"type": "string", "description": "Search query string"}
            },
            "required": ["query"]
        }),
    });
    dispatcher.register("read_note", Arc::new(read_note::ReadNoteTool::new(vault.clone())), ToolInfo {
        description: "Read the content of a single note".into(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "path": {"type": "string", "description": "Relative path to the note"}
            },
            "required": ["path"]
        }),
    });
    dispatcher.register("write_note", Arc::new(write_note::WriteNoteTool::new(vault.clone())), ToolInfo {
        description: "Create or overwrite a note with the given content".into(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "path": {"type": "string", "description": "Relative path to the note"},
                "content": {"type": "string", "description": "Markdown content to write"}
            },
            "required": ["path", "content"]
        }),
    });
    dispatcher.register("list_notes", Arc::new(list_notes::ListNotesTool::new(vault.clone())), ToolInfo {
        description: "List files and directories within a vault path".into(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "path": {"type": "string", "description": "Path relative to vault root (optional, defaults to vault root)"}
            }
        }),
    });
    dispatcher.register("get_tags", Arc::new(get_tags::GetTagsTool::new(vault.clone())), ToolInfo {
        description: "Get all tags across the vault with their occurrence counts".into(),
        input_schema: json!({
            "type": "object",
            "properties": {}
        }),
    });
    dispatcher.register("get_dates", Arc::new(get_dates::GetDatesTool::new(vault.clone())), ToolInfo {
        description: "Get all dates indexed across the vault, optionally filtered by a range".into(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "from": {"type": "string", "description": "Start date in DD.MM.YYYY format (optional)"},
                "to": {"type": "string", "description": "End date in DD.MM.YYYY format (optional)"}
            }
        }),
    });
    dispatcher.register("get_backlinks", Arc::new(get_backlinks::GetBacklinksTool::new(vault.clone())), ToolInfo {
        description: "Get all notes that link to a given target note".into(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "path": {"type": "string", "description": "Path to the target note"}
            },
            "required": ["path"]
        }),
    });
    dispatcher.register("get_outgoing_links", Arc::new(get_outgoing_links::GetOutgoingLinksTool::new(vault.clone())), ToolInfo {
        description: "Get all wiki-links from a given note to other notes".into(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "path": {"type": "string", "description": "Path to the source note"}
            },
            "required": ["path"]
        }),
    });
    dispatcher.register("resolve_link", Arc::new(resolve_link::ResolveLinkTool::new(vault.clone())), ToolInfo {
        description: "Resolve a [[wiki-link]] target to an actual file path".into(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "target": {"type": "string", "description": "Link target (e.g. note name without extension)"}
            },
            "required": ["target"]
        }),
    });
    #[cfg(feature = "git")]
    dispatcher.register("git_push", Arc::new(git_push::GitPushTool::new(vault.clone())), ToolInfo {
        description: "Stage all changes, commit, pull, and push to the remote git repository".into(),
        input_schema: json!({
            "type": "object",
            "properties": {}
        }),
    });
    #[cfg(feature = "git")]
    dispatcher.register("git_pull", Arc::new(git_pull::GitPullTool::new(vault.clone())), ToolInfo {
        description: "Pull latest changes from the remote git repository".into(),
        input_schema: json!({
            "type": "object",
            "properties": {}
        }),
    });
    dispatcher.register("validate_indexes", Arc::new(validate_indexes::ValidateIndexesTool::new(vault.clone())), ToolInfo {
        description: "Validate the integrity of all indexed data (notes, tags, dates)".into(),
        input_schema: json!({
            "type": "object",
            "properties": {}
        }),
    });
    dispatcher.register("reindex", Arc::new(reindex::ReindexTool::new(vault.clone())), ToolInfo {
        description: "Clear all indexes and rebuild them from scratch".into(),
        input_schema: json!({
            "type": "object",
            "properties": {}
        }),
    });
    dispatcher.register("get_diagnostics", Arc::new(get_diagnostics::GetDiagnosticsTool::new(vault)), ToolInfo {
        description: "Get diagnostics (broken links, parse errors) for all notes or a specific note".into(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "path": {"type": "string", "description": "Path to a specific note (optional, returns all diagnostics if omitted)"}
            }
        }),
    });
}
