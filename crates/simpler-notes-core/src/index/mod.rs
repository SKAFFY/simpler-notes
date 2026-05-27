mod tag_index;
mod date_index;

use std::path::PathBuf;
use std::fs;
use std::path::Path;
use dashmap::DashMap;
use crate::note::Note;

pub use tag_index::TagIndex;
pub use date_index::DateIndex;

#[derive(Debug, Default)]
pub struct FulltextIndex {
    terms: DashMap<String, Vec<PathBuf>>,
}

impl FulltextIndex {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&self, term: &str, path: PathBuf) {
        let lower = term.to_lowercase();
        self.terms.entry(lower)
            .or_insert_with(Vec::new)
            .push(path);
    }

    pub fn search(&self, term: &str) -> Vec<PathBuf> {
        self.terms.get(&term.to_lowercase())
            .map(|v| v.clone())
            .unwrap_or_default()
    }

    pub fn clear(&self) {
        self.terms.clear();
    }
}

#[derive(Debug, Default)]
pub struct ConcurrentIndex {
    pub tags: TagIndex,
    pub dates: DateIndex,
    pub fulltext: FulltextIndex,
    pub file_states: DashMap<PathBuf, u64>,
}

impl ConcurrentIndex {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn index_note(&self, note: &Note) {
        for tag in &note.metadata.tags {
            self.tags.add(tag, note.path.clone());
        }
        for date in &note.metadata.dates {
            self.dates.add(*date, note.path.clone());
        }
        for word in note.metadata.title.split_whitespace() {
            self.fulltext.add(word, note.path.clone());
        }
    }

    pub fn clear(&self) {
        self.tags.clear();
        self.dates.clear();
        self.fulltext.clear();
        self.file_states.clear();
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        fs::create_dir_all(path)
            .map_err(|e| format!("Failed to create index dir: {}", e))?;

        let tags_path = path.join("tags.json");
        let tag_data: Vec<(String, Vec<PathBuf>)> = {
            let tag_names = self.tags.all_tags();
            tag_names.into_iter().map(|tag| {
                let paths = self.tags.get(&tag);
                (tag, paths)
            }).collect()
        };
        fs::write(&tags_path, serde_json::to_string_pretty(&tag_data)
            .map_err(|e| format!("Serialize tags: {}", e))?)
            .map_err(|e| format!("Write tags: {}", e))?;

        let dates_path = path.join("dates.json");
        let date_data: Vec<(String, Vec<PathBuf>)> = {
            self.dates.all_dates().into_iter().map(|(date, paths)| {
                (date.format("%d.%m.%Y").to_string(), paths)
            }).collect()
        };
        fs::write(&dates_path, serde_json::to_string_pretty(&date_data)
            .map_err(|e| format!("Serialize dates: {}", e))?)
            .map_err(|e| format!("Write dates: {}", e))?;

        Ok(())
    }

    pub fn load(path: &Path) -> Result<Self, String> {
        let index = Self::new();

        let tags_path = path.join("tags.json");
        if tags_path.exists() {
            let data = fs::read_to_string(&tags_path)
                .map_err(|e| format!("Read tags: {}", e))?;
            let tag_data: Vec<(String, Vec<PathBuf>)> = serde_json::from_str(&data)
                .map_err(|e| format!("Parse tags: {}", e))?;
            for (tag, paths) in tag_data {
                for p in paths {
                    index.tags.add(&tag, p);
                }
            }
        }

        let dates_path = path.join("dates.json");
        if dates_path.exists() {
            let data = fs::read_to_string(&dates_path)
                .map_err(|e| format!("Read dates: {}", e))?;
            let date_data: Vec<(String, Vec<PathBuf>)> = serde_json::from_str(&data)
                .map_err(|e| format!("Parse dates: {}", e))?;
            for (date_str, paths) in date_data {
                if let Ok(date) = chrono::NaiveDate::parse_from_str(&date_str, "%d.%m.%Y") {
                    for p in paths {
                        index.dates.add(date, p);
                    }
                }
            }
        }

        Ok(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use chrono::NaiveDate;
    use crate::note::{Note, NoteMetadata};

    fn make_note(name: &str, tags: Vec<&str>, dates: Vec<&str>) -> Note {
        Note {
            path: PathBuf::from(format!("{}.md", name)),
            metadata: NoteMetadata {
                title: name.to_string(),
                links: vec![],
                tags: tags.into_iter().map(String::from).collect(),
                dates: dates.into_iter()
                    .filter_map(|d| NaiveDate::parse_from_str(d, "%d.%m.%Y").ok())
                    .collect(),
            },
        }
    }

    #[test]
    fn test_index_note_tags() {
        let idx = ConcurrentIndex::new();
        let note = make_note("test", vec!["project", "todo"], vec![]);
        idx.index_note(&note);

        assert!(!idx.tags.get("project").is_empty());
        assert!(!idx.tags.get("todo").is_empty());
    }

    #[test]
    fn test_index_note_dates() {
        let idx = ConcurrentIndex::new();
        let note = make_note("test", vec![], vec!["21.07.2003"]);
        idx.index_note(&note);

        let date = NaiveDate::from_ymd_opt(2003, 7, 21).unwrap();
        assert!(!idx.dates.get(date).is_empty());
    }

    #[test]
    fn test_index_clear() {
        let idx = ConcurrentIndex::new();
        let note = make_note("test", vec!["project"], vec![]);
        idx.index_note(&note);
        idx.clear();
        assert!(idx.tags.all_tags().is_empty());
    }

    #[test]
    fn test_index_save_and_load() {
        use std::fs;

        let dir = std::env::temp_dir().join("simpler_notes_index_test_persist");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let idx = ConcurrentIndex::new();
        let note = make_note("test", vec!["project"], vec!["21.07.2003"]);
        idx.index_note(&note);

        let save_result = idx.save(&dir);
        assert!(save_result.is_ok());
        assert!(dir.join("tags.json").exists());
        assert!(dir.join("dates.json").exists());

        let loaded = ConcurrentIndex::load(&dir).unwrap();
        assert!(!loaded.tags.get("project").is_empty());
        let date = chrono::NaiveDate::from_ymd_opt(2003, 7, 21).unwrap();
        assert!(!loaded.dates.get(date).is_empty());

        let _ = fs::remove_dir_all(&dir);
    }
}
