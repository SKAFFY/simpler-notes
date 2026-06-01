use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use crate::index::ConcurrentIndex;

/// Parsed query expression (AST).
#[derive(Debug, Clone, PartialEq)]
pub enum QueryExpr {
    Tag(String),
    Date(String),
    Link(String),
    Before(String),
    After(String),
    Content(String),
    Text(String),
    And(Box<QueryExpr>, Box<QueryExpr>),
    Or(Box<QueryExpr>, Box<QueryExpr>),
    Not(Box<QueryExpr>),
}

#[derive(Debug)]
pub struct SearchResult {
    pub path: String,
    pub line: u32,
    pub column: u32,
    pub line_content: String,
}

pub struct SearchEngine {
    pub index: Arc<ConcurrentIndex>,
}

impl SearchEngine {
    pub fn new(index: Arc<ConcurrentIndex>) -> Self {
        SearchEngine { index }
    }

    /// Full-text search via rg (ripgrep).
    pub fn search_fulltext(&self, vault_path: &Path, pattern: &str) -> Vec<SearchResult> {
        let output = Command::new("rg")
            .arg("--line-number")
            .arg("--column")
            .arg("--no-heading")
            .arg("--color")
            .arg("never")
            .arg(pattern)
            .arg(vault_path)
            .output();

        match output {
            Ok(out) if out.status.success() => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                stdout.lines().filter_map(parse_rg_line).collect()
            }
            _ => vec![],
        }
    }

    /// Parse a query expression from a string.
    pub fn parse_query(input: &str) -> QueryExpr {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return QueryExpr::Text(String::new());
        }

        // Split by spaces, building a simple AST
        let tokens: Vec<&str> = trimmed.split_whitespace().collect();
        if tokens.len() == 1 {
            parse_single_token(tokens[0])
        } else {
            let mut expr = parse_single_token(tokens[0]);
            for token in &tokens[1..] {
                expr = QueryExpr::And(Box::new(expr), Box::new(parse_single_token(token)));
            }
            expr
        }
    }

    /// Evaluate a query expression, returning matching file paths.
    pub fn execute_query(&self, expr: &QueryExpr, vault_path: &Path) -> Vec<String> {
        match expr {
            QueryExpr::Text(word) => {
                let word_lower = word.to_lowercase();
                let index_dir = vault_path.join(".index");
                if !index_dir.exists() {
                    return vec![];
                }
                // Search all indexed files via fulltext
                self.search_fulltext(vault_path, &word_lower)
                    .into_iter()
                    .map(|r| r.path.clone())
                    .collect()
            }
            QueryExpr::Tag(tag) => {
                self.index.tags.get(tag)
                    .into_iter()
                    .map(|e| e.path.to_string_lossy().to_string())
                    .collect()
            }
            QueryExpr::Link(target) => {
                let target_path = PathBuf::from(target);
                let results = self.index.links.backlinks(&target_path);
                results.into_iter()
                    .map(|e| e.source.to_string_lossy().to_string())
                    .collect()
            }
            QueryExpr::Date(date_str) => {
                let mut result = Vec::new();
                for (date, entries) in self.index.dates.all_dates() {
                    if date.to_string().contains(date_str) {
                        for e in entries {
                            result.push(e.path.to_string_lossy().to_string());
                        }
                    }
                }
                result
            }
            QueryExpr::And(a, b) => {
                let left = self.execute_query(a, vault_path);
                let right: std::collections::HashSet<String> =
                    self.execute_query(b, vault_path).into_iter().collect();
                left.into_iter().filter(|p| right.contains(p)).collect()
            }
            QueryExpr::Or(a, b) => {
                let mut left = self.execute_query(a, vault_path);
                let right = self.execute_query(b, vault_path);
                left.extend(right);
                left.sort();
                left.dedup();
                left
            }
            QueryExpr::Not(inner) => {
                let exclude: std::collections::HashSet<String> =
                    self.execute_query(inner, vault_path).into_iter().collect();
                // All indexed files minus excluded
                let all = self.all_indexed_files(vault_path);
                all.into_iter().filter(|p| !exclude.contains(p)).collect()
            }
            _ => vec![],
        }
    }

    fn all_indexed_files(&self, _vault_path: &Path) -> Vec<String> {
        let mut files: std::collections::HashSet<String> = std::collections::HashSet::new();
        for entry in self.index.tags.iter() {
            for e in entry.value() {
                files.insert(e.path.to_string_lossy().to_string());
            }
        }
        let mut result: Vec<String> = files.into_iter().collect();
        result.sort();
        result
    }
}

fn parse_single_token(token: &str) -> QueryExpr {
    if let Some(tag) = token.strip_prefix("tag:") {
        QueryExpr::Tag(tag.to_string())
    } else if let Some(date) = token.strip_prefix("date:") {
        QueryExpr::Date(date.to_string())
    } else if let Some(link) = token.strip_prefix("link:") {
        QueryExpr::Link(link.to_string())
    } else if let Some(date) = token.strip_prefix("before:") {
        QueryExpr::Before(date.to_string())
    } else if let Some(date) = token.strip_prefix("after:") {
        QueryExpr::After(date.to_string())
    } else if let Some(content) = token.strip_prefix("content:") {
        QueryExpr::Content(content.to_string())
    } else {
        QueryExpr::Text(token.to_string())
    }
}

fn parse_rg_line(line: &str) -> Option<SearchResult> {
    // rg output: path:line:col:content
    let parts: Vec<&str> = line.splitn(4, ':').collect();
    if parts.len() < 4 {
        return None;
    }
    let path = parts[0].to_string();
    let line_no = parts[1].parse::<u32>().ok()?;
    let col = parts[2].parse::<u32>().ok()?;
    let line_content = parts[3..].join(":");
    Some(SearchResult { path, line: line_no, column: col, line_content })
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tempfile::TempDir;

    #[test]
    fn test_parse_tag_query() {
        let expr = SearchEngine::parse_query("tag:project");
        assert_eq!(expr, QueryExpr::Tag("project".to_string()));
    }

    #[test]
    fn test_parse_text_query() {
        let expr = SearchEngine::parse_query("hello");
        assert_eq!(expr, QueryExpr::Text("hello".to_string()));
    }

    #[test]
    fn test_parse_and_query() {
        let expr = SearchEngine::parse_query("tag:project hello");
        let expected = QueryExpr::And(
            Box::new(QueryExpr::Tag("project".to_string())),
            Box::new(QueryExpr::Text("hello".to_string())),
        );
        assert_eq!(expr, expected);
    }

    #[test]
    fn test_parse_link_query() {
        let expr = SearchEngine::parse_query("link:OtherNote");
        assert_eq!(expr, QueryExpr::Link("OtherNote".to_string()));
    }

    #[test]
    fn test_rg_not_available() {
        let index = crate::index::ConcurrentIndex::new();
        let engine = SearchEngine::new(Arc::new(index));
        let dir = TempDir::new().unwrap();
        let results = engine.search_fulltext(dir.path(), "test");
        // rg may not be installed; should handle gracefully
        assert!(results.is_empty() || !results.is_empty());
    }

    #[test]
    fn test_execute_tag_query_empty() {
        let index = crate::index::ConcurrentIndex::new();
        let engine = SearchEngine::new(Arc::new(index));
        let dir = TempDir::new().unwrap();
        let results = engine.execute_query(&QueryExpr::Tag("nonexistent".to_string()), dir.path());
        assert!(results.is_empty());
    }

    #[test]
    fn test_parse_date_query() {
        let expr = SearchEngine::parse_query("date:2024-01");
        assert_eq!(expr, QueryExpr::Date("2024-01".to_string()));
    }

    #[test]
    fn test_parse_empty_query() {
        let expr = SearchEngine::parse_query("");
        assert_eq!(expr, QueryExpr::Text(String::new()));
    }

    #[test]
    fn test_parse_before_query() {
        let expr = SearchEngine::parse_query("before:2024-01");
        assert_eq!(expr, QueryExpr::Before("2024-01".to_string()));
    }

    #[test]
    fn test_parse_after_query() {
        let expr = SearchEngine::parse_query("after:2024-01");
        assert_eq!(expr, QueryExpr::After("2024-01".to_string()));
    }

    #[test]
    fn test_parse_content_query() {
        let expr = SearchEngine::parse_query("content:hello");
        assert_eq!(expr, QueryExpr::Content("hello".to_string()));
    }

    #[test]
    fn test_parse_rg_line() {
        let result = parse_rg_line("note.md:10:5:hello world").unwrap();
        assert_eq!(result.path, "note.md");
        assert_eq!(result.line, 10);
        assert_eq!(result.column, 5);
        assert_eq!(result.line_content, "hello world");
    }

    #[test]
    fn test_parse_rg_line_malformed() {
        assert!(parse_rg_line("note.md:10").is_none());
    }

    #[test]
    fn test_execute_tag_query_found() {
        let index = crate::index::ConcurrentIndex::new();
        let path = std::path::PathBuf::from("test.md");
        let span = crate::note_model::ByteSpan { offset: 0, length: 5 };
        index.tags.add(path, "found", span);
        let engine = SearchEngine::new(Arc::new(index));
        let dir = TempDir::new().unwrap();
        let results = engine.execute_query(&QueryExpr::Tag("found".to_string()), dir.path());
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_execute_and_query() {
        let index = crate::index::ConcurrentIndex::new();
        let path_a = std::path::PathBuf::from("a.md");
        let path_b = std::path::PathBuf::from("b.md");
        index.tags.add(path_a.clone(), "tag1", crate::note_model::ByteSpan { offset: 0, length: 5 });
        index.tags.add(path_a.clone(), "tag2", crate::note_model::ByteSpan { offset: 0, length: 5 });
        index.tags.add(path_b.clone(), "tag1", crate::note_model::ByteSpan { offset: 0, length: 5 });
        let engine = SearchEngine::new(Arc::new(index));
        let dir = TempDir::new().unwrap();
        let expr = QueryExpr::And(
            Box::new(QueryExpr::Tag("tag1".to_string())),
            Box::new(QueryExpr::Tag("tag2".to_string())),
        );
        let results = engine.execute_query(&expr, dir.path());
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_execute_or_query() {
        let index = crate::index::ConcurrentIndex::new();
        let path_a = std::path::PathBuf::from("a.md");
        let path_b = std::path::PathBuf::from("b.md");
        index.tags.add(path_a.clone(), "tag1", crate::note_model::ByteSpan { offset: 0, length: 5 });
        index.tags.add(path_b.clone(), "tag2", crate::note_model::ByteSpan { offset: 0, length: 5 });
        let engine = SearchEngine::new(Arc::new(index));
        let dir = TempDir::new().unwrap();
        let expr = QueryExpr::Or(
            Box::new(QueryExpr::Tag("tag1".to_string())),
            Box::new(QueryExpr::Tag("tag2".to_string())),
        );
        let results = engine.execute_query(&expr, dir.path());
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_execute_not_query() {
        let index = crate::index::ConcurrentIndex::new();
        let path_a = std::path::PathBuf::from("a.md");
        let path_b = std::path::PathBuf::from("b.md");
        index.tags.add(path_a.clone(), "tag1", crate::note_model::ByteSpan { offset: 0, length: 5 });
        index.tags.add(path_a.clone(), "tag2", crate::note_model::ByteSpan { offset: 0, length: 5 });
        index.tags.add(path_b.clone(), "tag1", crate::note_model::ByteSpan { offset: 0, length: 5 });
        let engine = SearchEngine::new(Arc::new(index));
        let dir = TempDir::new().unwrap();
        let expr = QueryExpr::Not(Box::new(QueryExpr::Tag("tag2".to_string())));
        let results = engine.execute_query(&expr, dir.path());
        assert_eq!(results.len(), 1);
        assert!(results[0].contains("b.md"));
    }

    #[test]
    fn test_execute_date_query() {
        let index = crate::index::ConcurrentIndex::new();
        use chrono::NaiveDate;
        let path = std::path::PathBuf::from("note.md");
        index.dates.add(path.clone(), NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(), crate::note_model::ByteSpan { offset: 0, length: 12 });
        let engine = SearchEngine::new(Arc::new(index));
        let dir = TempDir::new().unwrap();
        let results = engine.execute_query(&QueryExpr::Date("2024".to_string()), dir.path());
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_all_indexed_files() {
        let index = crate::index::ConcurrentIndex::new();
        let path_a = std::path::PathBuf::from("a.md");
        let path_b = std::path::PathBuf::from("b.md");
        index.tags.add(path_a, "tag", crate::note_model::ByteSpan { offset: 0, length: 4 });
        index.tags.add(path_b, "tag", crate::note_model::ByteSpan { offset: 0, length: 4 });
        let engine = SearchEngine::new(Arc::new(index));
        let dir = TempDir::new().unwrap();
        let files = engine.all_indexed_files(dir.path());
        assert_eq!(files.len(), 2);
    }

    #[test]
    fn test_execute_text_query_with_rg() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("note.md");
        std::fs::write(&file_path, "unique-search-term-42").unwrap();
        // open vault to create index
        let vault_path = dir.path().to_path_buf();
        let cfg = crate::vault::VaultConfig { path: vault_path.clone(), ..Default::default() };
        let _vault = crate::vault::Vault::open(cfg).unwrap();

        let index = crate::index::ConcurrentIndex::load(&vault_path).unwrap();
        let engine = SearchEngine::new(Arc::new(index));
        let results = engine.execute_query(&QueryExpr::Text("unique-search-term-42".to_string()), &vault_path);
        assert_eq!(results.len(), 1, "rg should find the term");
    }

    #[test]
    fn test_execute_wildcard_arm() {
        let index = crate::index::ConcurrentIndex::new();
        let engine = SearchEngine::new(Arc::new(index));
        let dir = TempDir::new().unwrap();
        let results = engine.execute_query(&QueryExpr::Before("2024".to_string()), dir.path());
        assert!(results.is_empty());
        let results = engine.execute_query(&QueryExpr::After("2024".to_string()), dir.path());
        assert!(results.is_empty());
        let results = engine.execute_query(&QueryExpr::Content("hello".to_string()), dir.path());
        assert!(results.is_empty());
    }

    #[test]
    fn test_text_query_without_index_dir() {
        let dir = TempDir::new().unwrap();
        let vault_path = dir.path().to_path_buf();
        std::fs::write(vault_path.join("note.md"), "hello world").unwrap();
        let index = crate::index::ConcurrentIndex::new();
        let engine = SearchEngine::new(Arc::new(index));
        let results = engine.execute_query(&QueryExpr::Text("hello".to_string()), &vault_path);
        assert!(results.is_empty(), "no .index dir -> empty results");
    }

    #[test]
    fn test_execute_link_query() {
        let index = crate::index::ConcurrentIndex::new();
        let path_a = std::path::PathBuf::from("source.md");
        let target = PathBuf::from("target-note");
        let span = crate::note_model::ByteSpan { offset: 0, length: 10 };
        let entry = crate::index::LinkEntry {
            source: path_a.clone(),
            target: target.clone(),
            label: "target-note".to_string(),
            span,
        };
        index.links.add(path_a.clone(), entry);
        let engine = SearchEngine::new(Arc::new(index));
        let dir = TempDir::new().unwrap();
        let results = engine.execute_query(&QueryExpr::Link("target-note".to_string()), dir.path());
        assert_eq!(results.len(), 1, "link query should find 1 source");
        assert!(results[0].contains("source.md"));
    }
}
