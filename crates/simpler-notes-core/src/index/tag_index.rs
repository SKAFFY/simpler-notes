use std::path::PathBuf;
use dashmap::DashMap;
#[derive(Debug, Default)]
pub struct TagIndex {
    tags: DashMap<String, Vec<PathBuf>>,
}

impl TagIndex {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&self, tag: &str, path: PathBuf) {
        self.tags.entry(tag.to_string())
            .or_insert_with(Vec::new)
            .push(path);
    }

    pub fn get(&self, tag: &str) -> Vec<PathBuf> {
        self.tags.get(tag)
            .map(|v| v.clone())
            .unwrap_or_default()
    }

    pub fn all_tags(&self) -> Vec<String> {
        self.tags.iter().map(|e| e.key().clone()).collect()
    }

    pub fn clear(&self) {
        self.tags.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tag_add_and_get() {
        let idx = TagIndex::new();
        idx.add("project", PathBuf::from("a.md"));
        idx.add("project", PathBuf::from("b.md"));
        idx.add("todo", PathBuf::from("a.md"));

        let project_files = idx.get("project");
        assert_eq!(project_files.len(), 2);

        let todo_files = idx.get("todo");
        assert_eq!(todo_files.len(), 1);

        let missing = idx.get("nonexistent");
        assert!(missing.is_empty());
    }

    #[test]
    fn test_tag_all_tags() {
        let idx = TagIndex::new();
        idx.add("project", PathBuf::from("a.md"));
        idx.add("todo", PathBuf::from("b.md"));
        idx.add("done", PathBuf::from("c.md"));

        let mut tags = idx.all_tags();
        tags.sort();
        assert_eq!(tags, vec!["done", "project", "todo"]);
    }

    #[test]
    fn test_tag_clear() {
        let idx = TagIndex::new();
        idx.add("project", PathBuf::from("a.md"));
        idx.clear();
        assert!(idx.all_tags().is_empty());
    }
}
