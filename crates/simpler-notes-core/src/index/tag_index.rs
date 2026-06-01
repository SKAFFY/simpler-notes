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

impl Default for TagIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl TagIndex {
    pub fn new() -> Self {
        TagIndex { tags: DashMap::new() }
    }

    pub fn add(&self, path: PathBuf, tag: &str, span: ByteSpan) {
        let mut entries = self.tags.entry(tag.to_string()).or_default();
        match entries.iter_mut().find(|e| e.path == path) {
            Some(entry) => entry.spans.push(span),
            None => entries.push(TagEntry { path, spans: vec![span] }),
        }
    }

    pub fn remove(&self, path: &Path, tag: &str) {
        if let Some(mut entries) = self.tags.get_mut(tag) {
            entries.retain(|e| e.path != path);
            if entries.is_empty() {
                drop(entries);
                self.tags.remove(tag);
            }
        }
    }

    pub fn get(&self, tag: &str) -> Vec<TagEntry> {
        self.tags.get(tag).map(|e| e.value().clone()).unwrap_or_default()
    }

    pub fn all_tags(&self) -> Vec<String> {
        self.tags.iter().map(|e| e.key().clone()).collect()
    }

    pub fn autocomplete(&self, prefix: &str) -> Vec<TagCompletion> {
        let lower = prefix.to_lowercase();
        let mut results: Vec<_> = self.tags.iter()
            .filter(|e| e.key().to_lowercase().starts_with(&lower))
            .map(|e| {
                let count: usize = e.value().iter().map(|en| en.spans.len()).sum();
                TagCompletion { name: e.key().clone(), count }
            })
            .collect();
        results.sort_by(|a, b| b.count.cmp(&a.count).then(a.name.cmp(&b.name)));
        results
    }

    pub fn fuzzy_search(&self, query: &str, max_results: usize) -> Vec<TagCompletion> {
        let q = query.to_lowercase();
        let mut results: Vec<_> = self.tags.iter()
            .filter(|e| {
                let name = e.key().to_lowercase();
                name.contains(&q) || q.contains(&name)
            })
            .map(|e| {
                let count: usize = e.value().iter().map(|en| en.spans.len()).sum();
                TagCompletion { name: e.key().clone(), count }
            })
            .collect();
        results.sort_by(|a, b| {
            let a_exact = a.name.to_lowercase() == q;
            let b_exact = b.name.to_lowercase() == q;
            b_exact.cmp(&a_exact).then(b.count.cmp(&a.count)).then(a.name.cmp(&b.name))
        });
        results.truncate(max_results);
        results
    }

    pub fn clear(&self) {
        self.tags.clear();
    }

    /// Remove all entries for a given file path.
    pub fn remove_file(&self, path: &Path) {
        let keys: Vec<String> = self.tags.iter()
            .filter(|e| e.value().iter().any(|entry| entry.path == path))
            .map(|e| e.key().clone())
            .collect();
        for tag in keys {
            self.remove(path, &tag);
        }
    }

    /// For serialization — iterate all entries
    pub fn iter(&self) -> dashmap::iter::Iter<'_, String, Vec<TagEntry>> {
        self.tags.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_get() {
        let index = TagIndex::new();
        let path = PathBuf::from("note.md");
        index.add(path.clone(), "project", ByteSpan { offset: 0, length: 8 });
        let entries = index.get("project");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].path, path);
    }

    #[test]
    fn test_add_multiple_spans() {
        let index = TagIndex::new();
        let path = PathBuf::from("note.md");
        index.add(path.clone(), "project", ByteSpan { offset: 0, length: 8 });
        index.add(path.clone(), "project", ByteSpan { offset: 100, length: 8 });
        let entries = index.get("project");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].spans.len(), 2);
    }

    #[test]
    fn test_remove() {
        let index = TagIndex::new();
        let path = PathBuf::from("note.md");
        index.add(path.clone(), "project", ByteSpan { offset: 0, length: 8 });
        index.remove(&path, "project");
        assert!(index.get("project").is_empty());
    }

    #[test]
    fn test_remove_partial() {
        let index = TagIndex::new();
        let a = PathBuf::from("a.md");
        let b = PathBuf::from("b.md");
        index.add(a.clone(), "tag", ByteSpan { offset: 0, length: 4 });
        index.add(b.clone(), "tag", ByteSpan { offset: 0, length: 4 });
        index.remove(&a, "tag");
        assert_eq!(index.get("tag").len(), 1);
        assert_eq!(index.get("tag")[0].path, b);
    }

    #[test]
    fn test_all_tags() {
        let index = TagIndex::new();
        let path = PathBuf::from("note.md");
        index.add(path.clone(), "project", ByteSpan { offset: 0, length: 8 });
        index.add(path.clone(), "todo", ByteSpan { offset: 10, length: 5 });
        let mut tags = index.all_tags();
        tags.sort();
        assert_eq!(tags, vec!["project", "todo"]);
    }

    #[test]
    fn test_autocomplete() {
        let index = TagIndex::new();
        let path = PathBuf::from("note.md");
        index.add(path.clone(), "project", ByteSpan { offset: 0, length: 8 });
        index.add(path.clone(), "project-management", ByteSpan { offset: 0, length: 18 });
        index.add(path.clone(), "todo", ByteSpan { offset: 0, length: 5 });
        let results = index.autocomplete("pro");
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|c| c.name.starts_with("pro")));
    }

    #[test]
    fn test_fuzzy_search() {
        let index = TagIndex::new();
        let path = PathBuf::from("note.md");
        index.add(path.clone(), "project-alpha", ByteSpan { offset: 0, length: 14 });
        index.add(path.clone(), "todo", ByteSpan { offset: 0, length: 5 });
        let results = index.fuzzy_search("alpha", 10);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "project-alpha");
    }

    #[test]
    fn test_clear() {
        let index = TagIndex::new();
        index.add(PathBuf::from("note.md"), "tag", ByteSpan { offset: 0, length: 4 });
        index.clear();
        assert!(index.all_tags().is_empty());
    }

    #[test]
    fn test_remove_file() {
        let index = TagIndex::new();
        let path = PathBuf::from("note.md");
        index.add(path.clone(), "project", ByteSpan { offset: 0, length: 8 });
        index.add(path.clone(), "todo", ByteSpan { offset: 10, length: 5 });
        index.add(PathBuf::from("other.md"), "project", ByteSpan { offset: 0, length: 8 });
        index.remove_file(&path);
        assert!(index.get("todo").is_empty());
        assert_eq!(index.get("project").len(), 1);
    }

    #[test]
    fn test_iter() {
        let index = TagIndex::new();
        let path = PathBuf::from("note.md");
        index.add(path.clone(), "tag", ByteSpan { offset: 0, length: 4 });
        let count: usize = index.iter().count();
        assert_eq!(count, 1);
    }
}
