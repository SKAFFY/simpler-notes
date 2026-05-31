use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc;
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

                        // Reindex
                        index.reindex_file(path, &content, vault_path);

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
}
