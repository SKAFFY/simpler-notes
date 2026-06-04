mod tag_index;
mod date_index;
mod link_index;

pub use tag_index::*;
pub use date_index::*;
pub use link_index::*;

use std::path::{Path, PathBuf};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use crate::diagnostics::Diagnostics;
use crate::note_model::ByteSpan;
use crate::util::normalize_path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileIndexState {
    pub tags: Vec<String>,
    pub dates: Vec<chrono::NaiveDate>,
}

pub struct ConcurrentIndex {
    pub tags: TagIndex,
    pub dates: DateIndex,
    pub links: LinkIndex,
    pub diagnostics: Diagnostics,
    pub file_states: DashMap<PathBuf, FileIndexState>,
}

impl Default for ConcurrentIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl ConcurrentIndex {
    pub fn new() -> Self {
        ConcurrentIndex {
            tags: TagIndex::new(),
            dates: DateIndex::new(),
            links: LinkIndex::new(),
            diagnostics: Diagnostics::new(),
            file_states: DashMap::new(),
        }
    }

    pub fn clear(&self) {
        self.tags.clear();
        self.dates.clear();
        self.links.clear();
        self.diagnostics.clear();
        self.file_states.clear();
    }

    /// Reindex one file — parse, update all indexes, run diagnostics.
    pub fn reindex_file(
        &self,
        path: &Path,
        content: &str,
        vault_path: &Path,
        filename_index: &std::collections::HashMap<String, Vec<PathBuf>>,
    ) {
        use crate::parser::parse_content;

        // Always remove old state before reindexing
        self.tags.remove_file(path);
        self.dates.remove_file(path);
        self.links.remove_file(path);
        self.diagnostics.remove(path);

        let result = parse_content(content);

        for tag_span in &result.tags {
            self.tags.add(
                path.to_path_buf(),
                &tag_span.name,
                ByteSpan { offset: tag_span.span.offset, length: tag_span.span.length },
            );
        }
        for date_span in &result.dates {
            self.dates.add(
                path.to_path_buf(),
                date_span.date,
                ByteSpan { offset: date_span.span.offset, length: date_span.span.length },
            );
        }
        for link_span in &result.links {
            let raw_target = PathBuf::from(&link_span.file_name);
            let resolved = if raw_target.is_absolute() {
                raw_target
            } else {
                path.parent().unwrap_or(Path::new("")).join(&raw_target)
            };
            let normalized = normalize_path(&resolved);
            let stem = normalized
                .file_stem()
                .unwrap_or(normalized.as_os_str())
                .to_string_lossy()
                .to_string();

            // Resolve stem to full path via filename_index:
            //   - 1 match → use that path (unambiguous)
            //   - 0 or >1 → use normalized path as fallback (broken/ambiguous)
            let target = filename_index
                .get(&stem)
                .and_then(|paths| if paths.len() == 1 { Some(paths[0].clone()) } else { None })
                .unwrap_or(normalized);

            let entry = LinkEntry {
                source: path.to_path_buf(),
                target,
                label: link_span.label.clone(),
                span: ByteSpan { offset: link_span.span.offset, length: link_span.span.length },
            };
            self.links.add(path.to_path_buf(), entry);
        }

        // Diagnostics
        self.diagnostics.check_file(path, content, vault_path, filename_index);

        self.file_states.insert(path.to_path_buf(), FileIndexState {
            tags: result.tags.iter().map(|t| t.name.clone()).collect(),
            dates: result.dates.iter().map(|d| d.date).collect(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_index() -> std::collections::HashMap<String, Vec<PathBuf>> {
        std::collections::HashMap::new()
    }

    #[test]
    fn test_clear_all() {
        let index = ConcurrentIndex::new();
        let path = PathBuf::from("test.md");
        index.tags.add(path.clone(), "tag", ByteSpan { offset: 0, length: 4 });
        index.dates.add(path.clone(), chrono::NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(), ByteSpan { offset: 0, length: 12 });
        let entry = LinkEntry {
            source: path.clone(),
            target: PathBuf::from("other.md"),
            label: "other".to_string(),
            span: ByteSpan { offset: 0, length: 10 },
        };
        index.links.add(path.clone(), entry);
        index.diagnostics.check_file(&path, "[[]]", &PathBuf::from("."), &empty_index());
        index.file_states.insert(path.clone(), FileIndexState { tags: vec!["tag".to_string()], dates: vec![] });
        index.clear();
        assert!(index.tags.all_tags().is_empty());
        assert!(index.dates.all_dates().is_empty());
        assert!(index.links.all_targets().is_empty());
        assert!(index.diagnostics.all().is_empty());
        assert!(index.file_states.is_empty());
    }

    #[test]
    fn test_reindex_replaces_state() {
        let index = ConcurrentIndex::new();
        let path = PathBuf::from("test.md");
        let content_a = "@tag1 !15.01.2024";
        index.reindex_file(&path, content_a, &PathBuf::from("."), &empty_index());
        assert!(index.tags.get("tag1").len() == 1);
        let content_b = "@tag2";
        index.reindex_file(&path, content_b, &PathBuf::from("."), &empty_index());
        assert!(index.tags.get("tag1").is_empty(), "old tag should be removed");
        assert!(index.tags.get("tag2").len() == 1, "new tag should be indexed");
    }

    #[test]
    fn test_table_driven_reindex_file() {
        struct Case {
            name: &'static str,
            content: &'static str,
            check: fn(&ConcurrentIndex, &mut Vec<String>),
        }

        let cases: Vec<Case> = vec![
            Case {
                name: "file with multiple links indexes all of them",
                content: "[[alpha]] and [[beta]] and [[gamma]]",
                check: |idx, errors| {
                    let targets = idx.links.all_targets();
                    if targets.len() != 3 {
                        errors.push(format!("expected 3 link targets, got {}", targets.len()));
                    }
                },
            },
            Case {
                name: "file with no links produces no link targets",
                content: "just plain text without any links",
                check: |idx, errors| {
                    let targets = idx.links.all_targets();
                    if targets.len() != 0 {
                        errors.push(format!("expected 0 link targets, got {}", targets.len()));
                    }
                },
            },
            Case {
                name: "reindexing replaces old links with new ones",
                content: "[[replaced]]",
                check: |idx, errors| {
                    let targets = idx.links.all_targets();
                    if targets.len() != 1 || targets[0] != PathBuf::from("replaced") {
                        errors.push(format!("expected [replaced], got {:?}", targets));
                    }
                },
            },
            Case {
                name: "file with multiple tags indexes all",
                content: "@work @project @urgent",
                check: |idx, errors| {
                    for tag in &["work", "project", "urgent"] {
                        if idx.tags.get(tag).is_empty() {
                            errors.push(format!("expected tag '{}' to be indexed", tag));
                        }
                    }
                },
            },
            Case {
                name: "file with dates indexes dates",
                content: "start !01.06.2026 end !15.06.2026",
                check: |idx, errors| {
                    let all = idx.dates.all_dates();
                    if all.len() != 2 {
                        errors.push(format!("expected 2 dates, got {}", all.len()));
                    }
                },
            },
            Case {
                name: "absolute path link is indexed by file_stem",
                content: "[[/tmp/abs-test]]",
                check: |idx, errors| {
                    let targets = idx.links.all_targets();
                    if targets.len() != 1 {
                        errors.push(format!("expected 1 target, got {}", targets.len()));
                    } else if !targets[0].to_string_lossy().ends_with("abs-test") {
                        errors.push(format!("expected path ending with abs-test, got {:?}", targets[0]));
                    }
                },
            },
        ];

        for (i, case) in cases.into_iter().enumerate() {
            let index = ConcurrentIndex::new();
            let path = PathBuf::from(format!("test_{}.md", i));
            index.reindex_file(&path, case.content, &PathBuf::from("."), &empty_index());
            let mut errors: Vec<String> = Vec::new();
            (case.check)(&index, &mut errors);
            assert!(errors.is_empty(), "case {} ({}): {}", i, case.name, errors.join("; "));
        }
    }

    #[test]
    fn test_table_driven_reindex_replaces_old_links() {
        struct Case {
            name: &'static str,
            first_content: &'static str,
            second_content: &'static str,
            check: fn(&ConcurrentIndex, &mut Vec<String>),
        }

        let cases: Vec<Case> = vec![
            Case {
                name: "reindex removes old links and adds new ones",
                first_content: "[[old-link]]",
                second_content: "[[new-link]]",
                check: |idx, errors| {
                    let old_bl = idx.links.backlinks(&PathBuf::from("old-link"));
                    if old_bl.len() != 0 {
                        errors.push("old link should be gone after reindex".into());
                    }
                    let new_bl = idx.links.backlinks(&PathBuf::from("new-link"));
                    if new_bl.len() != 1 {
                        errors.push("new link should exist after reindex".into());
                    }
                },
            },
            Case {
                name: "reindex with empty content clears all links",
                first_content: "[[link-a]] [[link-b]]",
                second_content: "",
                check: |idx, errors| {
                    if idx.links.all_targets().len() != 0 {
                        errors.push("reindexing to empty content should clear all links".into());
                    }
                },
            },
            Case {
                name: "reindex with empty content clears all tags",
                first_content: "@tag1 @tag2",
                second_content: "",
                check: |idx, errors| {
                    if !idx.tags.get("tag1").is_empty() {
                        errors.push("tag1 should be gone after reindex".into());
                    }
                    if !idx.tags.get("tag2").is_empty() {
                        errors.push("tag2 should be gone after reindex".into());
                    }
                },
            },
        ];

        for (i, case) in cases.into_iter().enumerate() {
            let index = ConcurrentIndex::new();
            let path = PathBuf::from("replace.md");
            index.reindex_file(&path, case.first_content, &PathBuf::from("."), &empty_index());
            index.reindex_file(&path, case.second_content, &PathBuf::from("."), &empty_index());
            let mut errors: Vec<String> = Vec::new();
            (case.check)(&index, &mut errors);
            assert!(errors.is_empty(), "case {} ({}): {}", i, case.name, errors.join("; "));
        }
    }
}
