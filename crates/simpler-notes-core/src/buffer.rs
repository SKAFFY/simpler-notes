use std::path::{Path, PathBuf};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct BufferEntry {
    pub content: String,
    pub saved_content: String,
    pub dirty: bool,
}

impl BufferEntry {
    pub fn new(content: String) -> Self {
        let saved = content.clone();
        BufferEntry { content, saved_content: saved, dirty: false }
    }

    pub fn set_content(&mut self, new_content: String) {
        self.content = new_content;
        self.dirty = self.content != self.saved_content;
    }

    pub fn mark_saved(&mut self) {
        self.saved_content = self.content.clone();
        self.dirty = false;
    }
}

pub struct Buffer {
    entries: HashMap<PathBuf, BufferEntry>,
}

impl Buffer {
    pub fn new() -> Self {
        Buffer { entries: HashMap::new() }
    }

    pub fn open(&mut self, path: &Path, content: String) {
        self.entries.insert(path.to_path_buf(), BufferEntry::new(content));
    }

    pub fn get(&self, path: &Path) -> Option<&BufferEntry> {
        self.entries.get(path)
    }

    pub fn get_mut(&mut self, path: &Path) -> Option<&mut BufferEntry> {
        self.entries.get_mut(path)
    }

    pub fn update(&mut self, path: &Path, new_content: String) {
        if let Some(entry) = self.entries.get_mut(path) {
            entry.set_content(new_content);
        } else {
            let entry = BufferEntry::new(new_content);
            // Newly created entry via update is dirty
            self.entries.insert(path.to_path_buf(), BufferEntry {
                content: entry.content,
                saved_content: String::new(),
                dirty: true,
            });
        }
    }

    pub fn save(&mut self, path: &Path) -> Option<String> {
        self.entries.get_mut(path).map(|entry| {
            entry.mark_saved();
            entry.content.clone()
        })
    }

    pub fn close(&mut self, path: &Path) {
        self.entries.remove(path);
    }

    pub fn is_dirty(&self, path: &Path) -> bool {
        self.entries.get(path).map(|e| e.dirty).unwrap_or(false)
    }

    pub fn dirty_files(&self) -> Vec<PathBuf> {
        self.entries.iter()
            .filter(|(_, e)| e.dirty)
            .map(|(p, _)| p.clone())
            .collect()
    }

    pub fn open_files(&self) -> Vec<PathBuf> {
        self.entries.keys().cloned().collect()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_and_get() {
        let mut buf = Buffer::new();
        buf.open(&PathBuf::from("test.md"), "hello".to_string());
        let entry = buf.get(&PathBuf::from("test.md")).unwrap();
        assert!(!entry.dirty);
        assert_eq!(entry.content, "hello");
    }

    #[test]
    fn test_update_marks_dirty() {
        let mut buf = Buffer::new();
        buf.open(&PathBuf::from("test.md"), "hello".to_string());
        buf.update(&PathBuf::from("test.md"), "world".to_string());
        assert!(buf.is_dirty(&PathBuf::from("test.md")));
    }

    #[test]
    fn test_save_clears_dirty() {
        let mut buf = Buffer::new();
        buf.open(&PathBuf::from("test.md"), "hello".to_string());
        buf.update(&PathBuf::from("test.md"), "world".to_string());
        buf.save(&PathBuf::from("test.md"));
        assert!(!buf.is_dirty(&PathBuf::from("test.md")));
    }

    #[test]
    fn test_close() {
        let mut buf = Buffer::new();
        buf.open(&PathBuf::from("test.md"), "hello".to_string());
        buf.close(&PathBuf::from("test.md"));
        assert!(buf.is_empty());
    }

    #[test]
    fn test_dirty_files() {
        let mut buf = Buffer::new();
        buf.open(&PathBuf::from("a.md"), "a".to_string());
        buf.open(&PathBuf::from("b.md"), "b".to_string());
        buf.update(&PathBuf::from("a.md"), "a2".to_string());
        let dirty = buf.dirty_files();
        assert_eq!(dirty.len(), 1);
        assert_eq!(dirty[0], PathBuf::from("a.md"));
    }

    #[test]
    fn test_open_files() {
        let mut buf = Buffer::new();
        buf.open(&PathBuf::from("a.md"), "a".to_string());
        buf.open(&PathBuf::from("b.md"), "b".to_string());
        assert_eq!(buf.open_files().len(), 2);
    }

    #[test]
    fn test_update_creates_if_not_open() {
        let mut buf = Buffer::new();
        buf.update(&PathBuf::from("new.md"), "fresh".to_string());
        assert_eq!(buf.len(), 1);
        assert!(buf.is_dirty(&PathBuf::from("new.md")));
    }

    #[test]
    fn test_get_nonexistent() {
        let buf = Buffer::new();
        assert!(buf.get(&PathBuf::from("nope.md")).is_none());
    }

    #[test]
    fn test_save_returns_content() {
        let mut buf = Buffer::new();
        buf.open(&PathBuf::from("test.md"), "hello".to_string());
        buf.update(&PathBuf::from("test.md"), "world".to_string());
        let saved = buf.save(&PathBuf::from("test.md"));
        assert_eq!(saved, Some("world".to_string()));
    }

    #[test]
    fn test_save_nonexistent() {
        let mut buf = Buffer::new();
        assert!(buf.save(&PathBuf::from("nope.md")).is_none());
    }

    #[test]
    fn test_get_mut() {
        let mut buf = Buffer::new();
        buf.open(&PathBuf::from("test.md"), "hello".to_string());
        let entry = buf.get_mut(&PathBuf::from("test.md")).unwrap();
        entry.set_content("world".to_string());
        assert!(entry.dirty);
        assert_eq!(entry.content, "world");
    }

    #[test]
    fn test_clear_buffer() {
        let mut buf = Buffer::new();
        buf.open(&PathBuf::from("a.md"), "a".to_string());
        buf.open(&PathBuf::from("b.md"), "b".to_string());
        assert!(!buf.is_empty());
        buf.clear();
        assert!(buf.is_empty());
        assert_eq!(buf.len(), 0);
    }
}
