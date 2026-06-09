use std::path::PathBuf;

use iced::widget::{button, column, container, scrollable, text};
use iced::Element;

use crate::app::{App, Message};

pub struct FileEntry {
    label: String,
    path: PathBuf,
    depth: usize,
}

fn collect_files(dir: PathBuf, vault_root: PathBuf, depth: usize, app: &App) -> Vec<FileEntry> {
    let mut entries = Vec::new();
    let Ok(read_dir) = std::fs::read_dir(&dir) else {
        return entries;
    };

    let mut dirs: Vec<PathBuf> = Vec::new();
    let mut files: Vec<PathBuf> = Vec::new();

    for entry in read_dir.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_dir() {
            dirs.push(path);
        } else if path.extension().and_then(|s| s.to_str()) == Some("md") {
            files.push(path);
        }
    }

    dirs.sort_by_key(|p| {
        p.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default()
    });
    files.sort_by_key(|p| {
        p.file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default()
    });

    for dir_path in dirs {
        let name = dir_path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        entries.push(FileEntry {
            label: format!("📁 {}", name),
            path: dir_path.clone(),
            depth,
        });
        let children = collect_files(dir_path, vault_root.clone(), depth + 1, app);
        entries.extend(children);
    }

    for file_path in files {
        let name = file_path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        let rel_path = file_path
            .strip_prefix(&vault_root)
            .unwrap_or(&file_path)
            .to_path_buf();

        let has_diag = app
            .vault
            .as_ref()
            .is_some_and(|v| !v.get_diagnostics(&rel_path).is_empty());
        let prefix = if has_diag { "⚠ " } else { "📄 " };

        entries.push(FileEntry {
            label: format!("{}{}", prefix, name),
            path: file_path,
            depth,
        });
    }

    entries
}

pub fn view(app: &App) -> Element<'_, Message> {
    let content: Element<'_, Message> = if let Some(ref vault) = app.vault {
        let root = vault.config.path.clone();
        let entries = collect_files(root.clone(), root, 0, app);
        let children: Vec<Element<'_, Message>> = entries
            .into_iter()
            .map(|entry| {
                let _indent = entry.depth * 16;
                let fp = entry.path;
                button(text(entry.label).size(13))
                    .on_press(Message::FileSelected(fp))
                    .into()
            })
            .collect();

        column(children).spacing(1).padding(4).into()
    } else {
        text("Open a vault to see files").size(13).into()
    };

    container(scrollable(content))
        .width(250)
        .height(iced::Fill)
        .into()
}
