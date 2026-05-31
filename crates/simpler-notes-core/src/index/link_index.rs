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

    pub fn add(&self, _source: PathBuf, entry: LinkEntry) {
        let target = entry.target.clone();
        self.backward.entry(target).or_default().push(entry);
    }

    pub fn remove_file(&self, path: &Path) {
        let mut empty_keys = Vec::new();
        for mut entry in self.backward.iter_mut() {
            entry.retain(|e| e.source != path);
            if entry.is_empty() {
                empty_keys.push(entry.key().clone());
            }
        }
        for key in empty_keys {
            self.backward.remove(&key);
        }
    }

    pub fn backlinks(&self, target: &Path) -> Vec<LinkEntry> {
        self.backward.get(target).map(|e| e.value().clone()).unwrap_or_default()
    }

    pub fn outgoing(&self, source: &Path) -> Vec<LinkEntry> {
        let mut result = Vec::new();
        for entry in self.backward.iter() {
            for e in entry.value() {
                if e.source == source {
                    result.push(e.clone());
                }
            }
        }
        result
    }

    pub fn clear(&self) {
        self.backward.clear();
    }

    pub fn all_targets(&self) -> Vec<PathBuf> {
        let mut targets: Vec<PathBuf> = self.backward.iter()
            .map(|e| e.key().clone())
            .collect();
        targets.sort();
        targets.dedup();
        targets
    }

    /// For serialization — iterate all entries
    pub fn iter(&self) -> dashmap::iter::Iter<'_, PathBuf, Vec<LinkEntry>> {
        self.backward.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_backlinks() {
        let index = LinkIndex::new();
        let source = PathBuf::from("a.md");
        let target = PathBuf::from("b.md");
        let entry = LinkEntry {
            source: source.clone(),
            target: target.clone(),
            label: "B".to_string(),
            span: ByteSpan { offset: 0, length: 10 },
        };
        index.add(source.clone(), entry);
        let backlinks = index.backlinks(&target);
        assert_eq!(backlinks.len(), 1);
        assert_eq!(backlinks[0].source, source);
    }

    #[test]
    fn test_outgoing() {
        let index = LinkIndex::new();
        let source = PathBuf::from("a.md");
        let target = PathBuf::from("b.md");
        let entry = LinkEntry {
            source: source.clone(),
            target: target.clone(),
            label: "B".to_string(),
            span: ByteSpan { offset: 0, length: 10 },
        };
        index.add(source.clone(), entry);
        let outgoing = index.outgoing(&source);
        assert_eq!(outgoing.len(), 1);
        assert_eq!(outgoing[0].target, target);
    }

    #[test]
    fn test_remove_file() {
        let index = LinkIndex::new();
        let source = PathBuf::from("a.md");
        let target = PathBuf::from("b.md");
        let entry = LinkEntry {
            source: source.clone(),
            target: target.clone(),
            label: "B".to_string(),
            span: ByteSpan { offset: 0, length: 10 },
        };
        index.add(source.clone(), entry);
        index.remove_file(&source);
        assert!(index.backlinks(&target).is_empty());
    }

    #[test]
    fn test_clear() {
        let index = LinkIndex::new();
        let entry = LinkEntry {
            source: PathBuf::from("a.md"),
            target: PathBuf::from("b.md"),
            label: "B".to_string(),
            span: ByteSpan { offset: 0, length: 10 },
        };
        index.add(PathBuf::from("a.md"), entry);
        index.clear();
        assert!(index.backlinks(&PathBuf::from("b.md")).is_empty());
    }
}
