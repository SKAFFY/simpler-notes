use std::path::PathBuf;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ByteSpan {
    pub offset: usize,
    pub length: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinkRef {
    pub file_name: String,
    pub label: String,
    pub span: ByteSpan,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TagRef {
    pub name: String,
    pub span: ByteSpan,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DateRef {
    pub date: NaiveDate,
    pub raw: String,
    pub span: ByteSpan,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NoteMetadata {
    pub links: Vec<LinkRef>,
    pub tags: Vec<TagRef>,
    pub dates: Vec<DateRef>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Note {
    pub path: PathBuf,
    pub metadata: NoteMetadata,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_byte_span_creation() {
        let span = ByteSpan { offset: 10, length: 5 };
        assert_eq!(span.offset, 10);
        assert_eq!(span.length, 5);
    }

    #[test]
    fn test_link_ref() {
        let link = LinkRef {
            file_name: "NoteName".to_string(),
            label: "Display Label".to_string(),
            span: ByteSpan { offset: 0, length: 20 },
        };
        assert_eq!(link.file_name, "NoteName");
        assert_eq!(link.label, "Display Label");
    }

    #[test]
    fn test_tag_ref() {
        let tag = TagRef {
            name: "project".to_string(),
            span: ByteSpan { offset: 5, length: 8 },
        };
        assert_eq!(tag.name, "project");
    }

    #[test]
    fn test_date_ref() {
        let date = DateRef {
            date: NaiveDate::from_ymd_opt(2003, 7, 21).unwrap(),
            raw: "!21.07.2003".to_string(),
            span: ByteSpan { offset: 10, length: 12 },
        };
        assert_eq!(date.date, NaiveDate::from_ymd_opt(2003, 7, 21).unwrap());
        assert_eq!(date.raw, "!21.07.2003");
    }

    #[test]
    fn test_note() {
        let note = Note {
            path: PathBuf::from("test.md"),
            metadata: NoteMetadata {
                links: vec![],
                tags: vec![],
                dates: vec![],
            },
        };
        assert_eq!(note.path, PathBuf::from("test.md"));
        assert!(note.metadata.links.is_empty());
    }
}
