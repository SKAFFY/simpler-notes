use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc;
use std::collections::HashMap;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher as NotifyWatcher};
use notify::event::EventKind;
use std::thread;
use parking_lot::RwLock;
use crate::buffer::Buffer;
use crate::index::ConcurrentIndex;

#[derive(Debug, Clone)]
pub enum VaultEvent {
    FileCreated(PathBuf),
    FileModified(PathBuf),
    FileDeleted(PathBuf),
    FileRenamed(PathBuf, PathBuf),
}

pub struct Watcher {
    _watcher: RecommendedWatcher,
}

impl Watcher {
    pub fn new(vault_path: &Path) -> Result<(Self, mpsc::Receiver<Result<Event, notify::Error>>), String> {
        let (tx, rx) = mpsc::channel();

        let mut watcher = RecommendedWatcher::new(
            move |res| {
                let _ = tx.send(res);
            },
            Config::default(),
        ).map_err(|e| e.to_string())?;

        watcher.watch(vault_path, RecursiveMode::Recursive)
            .map_err(|e| e.to_string())?;

        Ok((Watcher { _watcher: watcher }, rx))
    }

    /// Run a blocking event loop that reindexes files through the vault.
    pub fn run_event_loop(
        rx: mpsc::Receiver<Result<Event, notify::Error>>,
        vault_path: PathBuf,
        index: Arc<ConcurrentIndex>,
        buffer: Arc<RwLock<Buffer>>,
    ) {
        thread::spawn(move || {
            for event in rx {
                match event {
                    Ok(event) => {
                        Self::handle_event(&event, &vault_path, &index, &buffer);
                    }
                    Err(e) => {
                        eprintln!("Watcher error: {}", e);
                    }
                }
            }
        });
    }

    fn handle_event(
        event: &Event,
        vault_path: &Path,
        index: &Arc<ConcurrentIndex>,
        buffer: &Arc<RwLock<Buffer>>,
    ) {
        for path in &event.paths {
            let ext = path.extension()
                .and_then(|s| s.to_str())
                .unwrap_or("");
            if ext != "md" {
                continue;
            }

            match event.kind {
                EventKind::Create(_) | EventKind::Modify(_) => {
                    if let Ok(content) = std::fs::read_to_string(path) {
                        // Update buffer if file is open
                        let mut buf = buffer.write();
                        if buf.get(path).is_some() {
                            buf.update(path, content.clone());
                        }

                        // Build filename index for diagnostics
                        let mut filename_index: HashMap<String, Vec<PathBuf>> = HashMap::new();
                        if let Some(stem) = path.file_stem() {
                            filename_index.entry(stem.to_string_lossy().to_string())
                                .or_default()
                                .push(path.to_path_buf());
                        }

                        // Reindex
                        index.reindex_file(path, &content, vault_path, &filename_index);

                        // Persist
                        let _ = index.save(vault_path);
                    }
                }
                EventKind::Remove(_) => {
                    index.tags.remove_file(path);
                    index.dates.remove_file(path);
                    index.links.remove_file(path);
                    index.diagnostics.remove(path);

                    // Remove from buffer if open
                    let mut buf = buffer.write();
                    if buf.get(path).is_some() {
                        buf.close(path);
                    }

                    let _ = index.save(vault_path);
                }
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn test_watcher_creation() {
        let dir = TempDir::new().unwrap();
        let (_watcher, _rx) = Watcher::new(dir.path()).unwrap();
        // Just verify it doesn't panic
    }

    #[test]
    fn test_watcher_detects_create() {
        let dir = TempDir::new().unwrap();
        let vault_path = dir.path().to_path_buf();
        let index = Arc::new(ConcurrentIndex::new());
        let buffer = Arc::new(RwLock::new(Buffer::new()));

        let (_watcher, rx) = Watcher::new(&vault_path).unwrap();

        // Start event loop
        Watcher::run_event_loop(rx, vault_path.clone(), index.clone(), buffer.clone());

        // Create a markdown file
        let file_path = vault_path.join("test.md");
        fs::write(&file_path, "@tag").unwrap();

        // Give watcher time to process
        sleep(Duration::from_millis(200));

        // Check that the tag was indexed
        let tags = index.tags.get("tag");
        assert!(!tags.is_empty(), "Expected tag to be indexed after file creation");
    }

    #[test]
    fn test_watcher_modify_open_buffer() {
        let dir = TempDir::new().unwrap();
        let vault_path = dir.path().to_path_buf();
        let index = Arc::new(ConcurrentIndex::new());
        let buffer = Arc::new(RwLock::new(Buffer::new()));

        let file_path = vault_path.join("open.md");
        fs::write(&file_path, "original").unwrap();

        // Open file in buffer
        {
            let mut buf = buffer.write();
            buf.open(&file_path, "original".to_string());
        }

        let (_watcher, rx) = Watcher::new(&vault_path).unwrap();
        Watcher::run_event_loop(rx, vault_path.clone(), index.clone(), buffer.clone());

        // Modify the file
        fs::write(&file_path, "modified content").unwrap();
        sleep(Duration::from_millis(200));

        // Buffer should be updated
        let buf = buffer.read();
        let content = buf.get(&file_path);
        assert!(content.is_some(), "file should still be in buffer");
        assert_eq!(content.unwrap().content, "modified content", "buffer should reflect modified content");
    }

    #[test]
    fn test_watcher_remove_event() {
        let dir = TempDir::new().unwrap();
        let vault_path = dir.path().to_path_buf();
        let index = Arc::new(ConcurrentIndex::new());
        let buffer = Arc::new(RwLock::new(Buffer::new()));

        let file_path = vault_path.join("toremove.md");
        fs::write(&file_path, "@tag-removed").unwrap();

        // Manually index
        index.reindex_file(&file_path, "@tag-removed", &vault_path, &HashMap::new());
        assert!(!index.tags.get("tag-removed").is_empty(), "tag should be indexed");

        // Open in buffer
        {
            let mut buf = buffer.write();
            buf.open(&file_path, "@tag-removed".to_string());
            assert!(buf.get(&file_path).is_some(), "file should be open in buffer");
        }

        let (_watcher, rx) = Watcher::new(&vault_path).unwrap();
        Watcher::run_event_loop(rx, vault_path.clone(), index.clone(), buffer.clone());

        // Remove the file
        fs::remove_file(&file_path).unwrap();
        sleep(Duration::from_millis(200));

        // Verify index entries were removed
        assert!(index.tags.get("tag-removed").is_empty(), "tag should be removed on delete");
        // Verify buffer entry was closed
        let buf = buffer.read();
        assert!(buf.get(&file_path).is_none(), "buffer should close on delete");
    }

    #[test]
    fn test_watcher_other_event_kind() {
        let dir = TempDir::new().unwrap();
        let vault_path = dir.path().to_path_buf();
        let index = Arc::new(ConcurrentIndex::new());
        let buffer = Arc::new(RwLock::new(Buffer::new()));

        let file_path = vault_path.join("other.md");
        fs::write(&file_path, "test").unwrap();

        let (tx, rx) = std::sync::mpsc::channel();
        Watcher::run_event_loop(rx, vault_path.clone(), index.clone(), buffer.clone());

        // Send an Other event kind — should hit the _ => arm
        use notify::event::EventKind;
        let event = Event {
            kind: EventKind::Other,
            paths: vec![file_path.clone()],
            ..Default::default()
        };
        tx.send(Ok(event)).unwrap();
        sleep(Duration::from_millis(50));

        // File should not be indexed since Other is skipped
        assert!(index.tags.all_tags().is_empty(), "Other event should not index");
    }
}
