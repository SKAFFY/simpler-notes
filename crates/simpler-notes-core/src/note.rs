use std::path::PathBuf;
use chrono::NaiveDate;

#[derive(Debug, Clone)]
pub struct Note {
    pub path: PathBuf,
    pub metadata: NoteMetadata,
}

#[derive(Debug, Clone)]
pub struct NoteMetadata {
    pub title: String,
    pub links: Vec<String>,
    pub tags: Vec<String>,
    pub dates: Vec<NaiveDate>,
}
