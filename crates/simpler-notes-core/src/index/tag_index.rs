use std::path::{Path, PathBuf};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use crate::note_model::ByteSpan;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagEntry {
    pub path: PathBuf,
    pub spans: Vec<ByteSpan>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagCompletion {
    pub name: String,
    pub count: usize,
}

pub struct TagIndex {
    tags: DashMap<String, Vec<TagEntry>>,
}

impl TagIndex {
    pub fn new() -> Self {
        TagIndex { tags: DashMap::new() }
    }

    pub fn clear(&self) {
        self.tags.clear();
    }
}
