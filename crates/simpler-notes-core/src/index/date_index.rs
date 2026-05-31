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

    pub fn add(&self, path: PathBuf, date: NaiveDate, span: ByteSpan) {
        let mut entries = self.dates.entry(date).or_default();
        match entries.iter_mut().find(|e| e.path == path) {
            Some(e) => e.spans.push(span),
            None => entries.push(DateEntry { path, spans: vec![span] }),
        }
    }

    pub fn remove(&self, path: &Path, date: NaiveDate) {
        if let Some(mut entries) = self.dates.get_mut(&date) {
            entries.retain(|e| e.path != path);
            if entries.is_empty() {
                drop(entries);
                self.dates.remove(&date);
            }
        }
    }

    pub fn get(&self, date: NaiveDate) -> Vec<DateEntry> {
        self.dates.get(&date).map(|e| e.value().clone()).unwrap_or_default()
    }

    pub fn get_range(&self, from: NaiveDate, to: NaiveDate) -> Vec<(NaiveDate, Vec<DateEntry>)> {
        let mut result: Vec<_> = self.dates.iter()
            .filter(|e| *e.key() >= from && *e.key() <= to)
            .map(|e| (*e.key(), e.value().clone()))
            .collect();
        result.sort_by(|a, b| a.0.cmp(&b.0));
        result
    }

    pub fn all_dates(&self) -> Vec<(NaiveDate, Vec<DateEntry>)> {
        let mut result: Vec<_> = self.dates.iter()
            .map(|e| (*e.key(), e.value().clone()))
            .collect();
        result.sort_by(|a, b| a.0.cmp(&b.0));
        result
    }

    pub fn clear(&self) {
        self.dates.clear();
    }

    /// Remove all entries for a given file path.
    pub fn remove_file(&self, path: &Path) {
        let keys: Vec<NaiveDate> = self.dates.iter()
            .filter(|e| e.value().iter().any(|entry| entry.path == path))
            .map(|e| *e.key())
            .collect();
        for date in keys {
            self.remove(path, date);
        }
    }

    /// For serialization — iterate all entries
    pub fn iter(&self) -> dashmap::iter::Iter<'_, NaiveDate, Vec<DateEntry>> {
        self.dates.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    #[test]
    fn test_add_and_get() {
        let index = DateIndex::new();
        let path = PathBuf::from("note.md");
        index.add(path.clone(), d(2024, 1, 15), ByteSpan { offset: 0, length: 12 });
        let entries = index.get(d(2024, 1, 15));
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn test_get_range() {
        let index = DateIndex::new();
        let path = PathBuf::from("note.md");
        index.add(path.clone(), d(2024, 1, 15), ByteSpan { offset: 0, length: 12 });
        index.add(path.clone(), d(2024, 3, 20), ByteSpan { offset: 0, length: 12 });
        let range = index.get_range(d(2024, 1, 1), d(2024, 2, 1));
        assert_eq!(range.len(), 1);
        assert_eq!(range[0].0, d(2024, 1, 15));
    }

    #[test]
    fn test_remove() {
        let index = DateIndex::new();
        let path = PathBuf::from("note.md");
        index.add(path.clone(), d(2024, 1, 15), ByteSpan { offset: 0, length: 12 });
        index.remove(&path, d(2024, 1, 15));
        assert!(index.get(d(2024, 1, 15)).is_empty());
    }

    #[test]
    fn test_all_dates() {
        let index = DateIndex::new();
        let path = PathBuf::from("note.md");
        index.add(path.clone(), d(2024, 1, 15), ByteSpan { offset: 0, length: 12 });
        index.add(path.clone(), d(2024, 3, 20), ByteSpan { offset: 0, length: 12 });
        assert_eq!(index.all_dates().len(), 2);
    }

    #[test]
    fn test_clear() {
        let index = DateIndex::new();
        index.add(PathBuf::from("note.md"), d(2024, 1, 15), ByteSpan { offset: 0, length: 12 });
        index.clear();
        assert!(index.all_dates().is_empty());
    }
}
