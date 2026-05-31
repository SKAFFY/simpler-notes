use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct InputSchema {
    #[serde(rename = "type")]
    pub type_: String,
    pub properties: Vec<PropertyDefinition>,
    pub required: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct PropertyDefinition {
    pub name: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub description: String,
}

#[derive(Debug, Serialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: InputSchema,
}

pub fn all_tools() -> Vec<ToolDefinition> {
    use ToolDefinition as T;

    vec![
        T {
            name: "search_notes".into(),
            description: "Search notes using query language. Supports: tags contain \"tag\", date before DD.MM.YYYY, date after DD.MM.YYYY, and/or combinators, plain text search.".into(),
            input_schema: InputSchema {
                type_: "object".into(),
                properties: vec![PropertyDefinition {
                    name: "query".into(),
                    type_: "string".into(),
                    description: "Search query".into(),
                }],
                required: vec!["query".into()],
            },
        },
        T {
            name: "read_note".into(),
            description: "Read the content of a note file.".into(),
            input_schema: InputSchema {
                type_: "object".into(),
                properties: vec![PropertyDefinition {
                    name: "path".into(),
                    type_: "string".into(),
                    description: "Relative path to the note file from the vault root.".into(),
                }],
                required: vec!["path".into()],
            },
        },
        T {
            name: "write_note".into(),
            description: "Create or update a note file.".into(),
            input_schema: InputSchema {
                type_: "object".into(),
                properties: vec![
                    PropertyDefinition {
                        name: "path".into(),
                        type_: "string".into(),
                        description: "Relative path to the note file from the vault root.".into(),
                    },
                    PropertyDefinition {
                        name: "content".into(),
                        type_: "string".into(),
                        description: "Markdown content of the note.".into(),
                    },
                ],
                required: vec!["path".into(), "content".into()],
            },
        },
        T {
            name: "list_notes".into(),
            description: "List all markdown notes in the vault, optionally filtered by a subdirectory.".into(),
            input_schema: InputSchema {
                type_: "object".into(),
                properties: vec![PropertyDefinition {
                    name: "path".into(),
                    type_: "string".into(),
                    description: "Optional subdirectory path to list notes from.".into(),
                }],
                required: vec![],
            },
        },
        T {
            name: "get_tags".into(),
            description: "Get all tags used across notes in the vault.".into(),
            input_schema: InputSchema {
                type_: "object".into(),
                properties: vec![],
                required: vec![],
            },
        },
        T {
            name: "get_dates".into(),
            description: "Get all dates found in notes, grouped by date.".into(),
            input_schema: InputSchema {
                type_: "object".into(),
                properties: vec![],
                required: vec![],
            },
        },
        T {
            name: "validate_indexes".into(),
            description: "Validate index integrity and return counts of notes, tags, and dates.".into(),
            input_schema: InputSchema {
                type_: "object".into(),
                properties: vec![],
                required: vec![],
            },
        },
        T {
            name: "git_push".into(),
            description: "Push local commits to remote git repository. Requires git feature to be enabled.".into(),
            input_schema: InputSchema {
                type_: "object".into(),
                properties: vec![],
                required: vec![],
            },
        },
        T {
            name: "git_pull".into(),
            description: "Pull latest changes from remote git repository. Requires git feature to be enabled.".into(),
            input_schema: InputSchema {
                type_: "object".into(),
                properties: vec![],
                required: vec![],
            },
        },
    ]
}

pub fn find_tool(name: &str) -> Option<ToolDefinition> {
    all_tools().into_iter().find(|t| t.name == name)
}

pub fn validate_required(tool: &ToolDefinition, args: &serde_json::Value) -> Result<(), String> {
    for field in &tool.input_schema.required {
        if !args.get(field).and_then(|v| v.as_str()).map_or(false, |s| !s.is_empty()) {
            return Err(format!("Missing required field: {}", field));
        }
    }
    Ok(())
}
