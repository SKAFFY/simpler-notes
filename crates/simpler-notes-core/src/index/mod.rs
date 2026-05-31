mod tag_index;
mod date_index;
mod link_index;

pub use tag_index::*;
pub use date_index::*;
pub use link_index::*;

use std::path::PathBuf;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileIndexState {
    pub tags: Vec<String>,
    pub dates: Vec<chrono::NaiveDate>,
}

pub struct ConcurrentIndex {
    pub tags: TagIndex,
    pub dates: DateIndex,
    pub links: LinkIndex,
    pub file_states: DashMap<PathBuf, FileIndexState>,
}

impl ConcurrentIndex {
    pub fn new() -> Self {
        ConcurrentIndex {
            tags: TagIndex::new(),
            dates: DateIndex::new(),
            links: LinkIndex::new(),
            file_states: DashMap::new(),
        }
    }

    pub fn clear(&self) {
        self.tags.clear();
        self.dates.clear();
        self.links.clear();
        self.file_states.clear();
    }
}
