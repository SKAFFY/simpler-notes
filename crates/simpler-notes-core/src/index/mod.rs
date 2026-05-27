mod tag_index;
mod date_index;

use std::path::PathBuf;
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
}
