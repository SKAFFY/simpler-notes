use std::path::Path;
use std::sync::mpsc;
use std::thread;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher as NotifyWatcher};

#[derive(Debug, Clone)]
pub enum FileEvent {
    Created(String),
    Modified(String),
    Deleted(String),
}

pub struct FileWatcher {
    pub receiver: mpsc::Receiver<FileEvent>,
    _handle: thread::JoinHandle<()>,
}

impl FileWatcher {
    pub fn start(path: &Path) -> Result<Self, String> {
        let (tx, rx) = mpsc::channel();
        let watch_path = path.to_path_buf();

        let handle = thread::spawn(move || {
            let (notify_tx, notify_rx) = mpsc::channel::<Event>();

            let mut watcher = match RecommendedWatcher::new(
                move |res: Result<Event, notify::Error>| {
                    if let Ok(event) = res {
                        if matches!(event.kind, EventKind::Access(_)) {
                            return;
                        }
                        let _ = notify_tx.send(event);
                    }
                },
                Config::default(),
            ) {
                Ok(w) => w,
                Err(e) => {
                    eprintln!("Failed to create watcher: {}", e);
                    return;
                }
            };

            if let Err(e) = watcher.watch(&watch_path, RecursiveMode::Recursive) {
                eprintln!("Failed to watch path: {}", e);
                return;
            }

            for event in notify_rx {
                for path in event.paths {
                    let path_str = path.to_string_lossy().to_string();
                    if path_str.contains("/.git/") || path_str.contains("/.index/") {
                        continue;
                    }

                    let event = match event.kind {
                        EventKind::Create(_) => FileEvent::Created(path_str),
                        EventKind::Modify(_) => FileEvent::Modified(path_str),
                        EventKind::Remove(_) => FileEvent::Deleted(path_str),
                        _ => continue,
                    };

                    if tx.send(event).is_err() {
                        break;
                    }
                }
            }
        });

        Ok(Self {
            receiver: rx,
            _handle: handle,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::Duration;

    #[test]
    fn test_watcher_detects_created_file() {
        let dir = std::env::temp_dir().join("simpler_notes_watcher_test");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let watcher = FileWatcher::start(&dir).unwrap();

        fs::write(dir.join("test.md"), "hello").unwrap();

        let event = watcher.receiver.recv_timeout(Duration::from_secs(3));
        match event {
            Ok(FileEvent::Created(path)) => {
                assert!(path.ends_with("test.md"));
            }
            other => {
                match other {
                    Ok(FileEvent::Modified(path)) => {
                        assert!(path.ends_with("test.md"));
                    }
                    _ => {}
                }
            }
        }

        let _ = fs::remove_dir_all(&dir);
    }
}
