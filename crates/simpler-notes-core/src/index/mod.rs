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
    pub fn reindex_file(&self, path: &Path, content: &str, vault_path: &Path) {
        use crate::parser::parse_content;

        if let Some(old_state) = self.file_states.get(path) {
            for tag in &old_state.tags {
                self.tags.remove(path, tag);
            }
            for date in &old_state.dates {
                self.dates.remove(path, *date);
            }
        }
        self.links.remove_file(path);

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
            let target = PathBuf::from(&link_span.file_name);
            let entry = LinkEntry {
                source: path.to_path_buf(),
                target: target.clone(),
                label: link_span.label.clone(),
                span: ByteSpan { offset: link_span.span.offset, length: link_span.span.length },
            };
            self.links.add(path.to_path_buf(), entry);
        }

        // Diagnostics
        self.diagnostics.remove(path);
        self.diagnostics.check_file(path, content, vault_path);

        self.file_states.insert(path.to_path_buf(), FileIndexState {
            tags: result.tags.iter().map(|t| t.name.clone()).collect(),
            dates: result.dates.iter().map(|d| d.date).collect(),
        });
    }
}
