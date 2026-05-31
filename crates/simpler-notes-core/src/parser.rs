use std::path::Path;

use chrono::NaiveDate;
use regex::Regex;

use crate::note::{Note, NoteMetadata};

pub fn parse_note(path: &Path, content: &str) -> Note {
    let title = extract_title(path, content);
    let links = extract_links(content);
    let tags = extract_tags(content);
    let dates = extract_dates(content);

    Note {
        path: path.to_path_buf(),
        metadata: NoteMetadata {
            title,
            links,
            tags,
            dates,
        },
    }
}

pub fn extract_title(path: &Path, content: &str) -> String {
    let re = Regex::new(r"(?m)^# (.+)$").unwrap();
    if let Some(cap) = re.captures(content) {
        return cap[1].trim().to_string();
    }
    path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string()
}

pub fn extract_links(content: &str) -> Vec<String> {
    let re = Regex::new(r"\[\[([^\[\]]+?)(?:\|[^\[\]]*)?\]\]").unwrap();
    re.captures_iter(content)
        .map(|cap| cap[1].trim().to_string())
        .collect()
}

pub fn extract_tags(content: &str) -> Vec<String> {
    let re = Regex::new(r"(?:^|\s)@([a-zA-Zа-яА-Я0-9_\-]+)").unwrap();
    let mut tags: Vec<String> = re
        .captures_iter(content)
        .map(|cap| cap[1].to_string())
        .collect();
    tags.sort();
    tags.dedup();
    tags
}

pub fn extract_dates(content: &str) -> Vec<NaiveDate> {
    let re = Regex::new(r"\b(\d{2})\.(\d{2})\.(\d{4})\b").unwrap();
    re.captures_iter(content)
        .filter_map(|cap| {
            let day = cap[1].parse::<u32>().ok()?;
            let month = cap[2].parse::<u32>().ok()?;
            let year = cap[3].parse::<i32>().ok()?;
            NaiveDate::from_ymd_opt(year, month, day)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_extract_title_from_h1() {
        let content = "# My Note\n\nBody";
        assert_eq!(extract_title(Path::new("any.md"), content), "My Note");
    }

    #[test]
    fn test_extract_title_fallback_to_filename() {
        let content = "No heading";
        assert_eq!(extract_title(Path::new("hello.md"), content), "hello");
    }

    #[test]
    fn test_extract_links_simple() {
        let content = "See [[Other Note]] and [[Project|Alias]]";
        let links = extract_links(content);
        assert_eq!(links, vec!["Other Note", "Project"]);
    }

    #[test]
    fn test_extract_tags() {
        let content = "@project this is @todo and @project";
        let tags = extract_tags(content);
        assert_eq!(tags, vec!["project", "todo"]);
    }

    #[test]
    fn test_extract_dates() {
        let content = "Deadline: 21.07.2003 and 01.01.2024";
        let dates = extract_dates(content);
        assert_eq!(dates.len(), 2);
        assert_eq!(dates[0], NaiveDate::from_ymd_opt(2003, 7, 21).unwrap());
    }

    #[test]
    fn test_extract_invalid_date_ignored() {
        let content = "32.13.2000";
        let dates = extract_dates(content);
        assert!(dates.is_empty());
    }

    #[test]
    fn test_extract_title_fallback_on_empty() {
        let content = "";
        assert_eq!(extract_title(Path::new("untitled.md"), content), "untitled");
    }

    #[test]
    fn test_extract_no_links() {
        let content = "No brackets here";
        let links = extract_links(content);
        assert!(links.is_empty());
    }

    #[test]
    fn test_extract_tags_no_duplicates() {
        let content = "@tag1 @tag1 @tag2";
        let tags = extract_tags(content);
        assert_eq!(tags, vec!["tag1", "tag2"]);
    }

    #[test]
    fn test_extract_tags_no_false_positive() {
        let content = "## Heading not a tag";
        let tags = extract_tags(content);
        assert!(!tags.contains(&"Heading".to_string()));
    }
}
