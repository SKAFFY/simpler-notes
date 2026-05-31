use std::path::{Path, PathBuf};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use crate::note_model::ByteSpan;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkEntry {
    pub source: PathBuf,
    pub target: PathBuf,
    pub label: String,
    pub span: ByteSpan,
}

pub struct LinkIndex {
    backward: DashMap<PathBuf, Vec<LinkEntry>>,
}

impl LinkIndex {
    pub fn new() -> Self {
        LinkIndex { backward: DashMap::new() }
    }

    pub fn clear(&self) {
        self.backward.clear();
    }
}
