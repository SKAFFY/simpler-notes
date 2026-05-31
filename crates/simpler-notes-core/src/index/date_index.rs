use std::path::{Path, PathBuf};
use chrono::NaiveDate;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use crate::note_model::ByteSpan;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateEntry {
    pub path: PathBuf,
    pub spans: Vec<ByteSpan>,
}

pub struct DateIndex {
    dates: DashMap<NaiveDate, Vec<DateEntry>>,
}

impl DateIndex {
    pub fn new() -> Self {
        DateIndex { dates: DashMap::new() }
    }

    pub fn clear(&self) {
        self.dates.clear();
    }
}
