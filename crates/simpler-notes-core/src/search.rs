use std::path::PathBuf;
use std::collections::HashSet;
use chrono::NaiveDate;
use crate::index::ConcurrentIndex;

#[derive(Debug, Clone, PartialEq)]
pub enum Query {
    TagsContain(String),
    DateBefore(NaiveDate),
    DateAfter(NaiveDate),
    Text(String),
    And(Box<Query>, Box<Query>),
    Or(Box<Query>, Box<Query>),
}

#[derive(Debug)]
pub struct SearchResult {
    pub path: PathBuf,
    pub title: String,
}

pub fn parse_query(input: &str) -> Result<Query, String> {
    let trimmed = input.trim();

    if let Some(pos) = trimmed.find(" and ") {
        let left = parse_query(&trimmed[..pos])?;
        let right = parse_query(&trimmed[pos + 5..])?;
        return Ok(Query::And(Box::new(left), Box::new(right)));
    }

    if let Some(pos) = trimmed.find(" or ") {
        let left = parse_query(&trimmed[..pos])?;
        let right = parse_query(&trimmed[pos + 4..])?;
        return Ok(Query::Or(Box::new(left), Box::new(right)));
    }

    if let Some(remainder) = trimmed.strip_prefix("tags contain ") {
        let remainder = remainder.trim();
        let tag = if remainder.starts_with('"') {
            remainder[1..].split('"').next().ok_or("Missing closing quote")?.to_string()
        } else {
            remainder.split_whitespace().next().ok_or("Missing tag value")?.to_string()
        };
        return Ok(Query::TagsContain(tag));
    }

    if let Some(remainder) = trimmed.strip_prefix("date before ") {
        let date_str = remainder.trim();
        let date = NaiveDate::parse_from_str(date_str, "%d.%m.%Y")
            .map_err(|e| format!("Invalid date '{}': {}", date_str, e))?;
        return Ok(Query::DateBefore(date));
    }

    if let Some(remainder) = trimmed.strip_prefix("date after ") {
        let date_str = remainder.trim();
        let date = NaiveDate::parse_from_str(date_str, "%d.%m.%Y")
            .map_err(|e| format!("Invalid date '{}': {}", date_str, e))?;
        return Ok(Query::DateAfter(date));
    }

    Ok(Query::Text(trimmed.to_string()))
}

pub fn execute_query(index: &ConcurrentIndex, query: &Query) -> Vec<PathBuf> {
    match query {
        Query::TagsContain(tag) => index.tags.get(tag),
        Query::DateBefore(target) => {
            let mut result = Vec::new();
            for (date, paths) in index.dates.all_dates() {
                if date <= *target {
                    result.extend(paths);
                }
            }
            result.sort();
            result.dedup();
            result
        }
        Query::DateAfter(target) => {
            let mut result = Vec::new();
            for (date, paths) in index.dates.all_dates() {
                if date >= *target {
                    result.extend(paths);
                }
            }
            result.sort();
            result.dedup();
            result
        }
        Query::Text(term) => index.fulltext.search(term),
        Query::And(left, right) => {
            let left_results: HashSet<_> = execute_query(index, left).into_iter().collect();
            let right_results: HashSet<_> = execute_query(index, right).into_iter().collect();
            let mut intersection: Vec<_> = left_results.intersection(&right_results).cloned().collect();
            intersection.sort();
            intersection
        }
        Query::Or(left, right) => {
            let mut left_results = execute_query(index, left);
            let mut right_results = execute_query(index, right);
            left_results.append(&mut right_results);
            left_results.sort();
            left_results.dedup();
            left_results
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn test_parse_tags_contain() {
        let q = parse_query("tags contain \"project\"").unwrap();
        assert_eq!(q, Query::TagsContain("project".to_string()));
    }

    #[test]
    fn test_parse_date_before() {
        let q = parse_query("date before 01.01.2024").unwrap();
        assert_eq!(q, Query::DateBefore(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()));
    }

    #[test]
    fn test_parse_date_after() {
        let q = parse_query("date after 15.06.2024").unwrap();
        assert_eq!(q, Query::DateAfter(NaiveDate::from_ymd_opt(2024, 6, 15).unwrap()));
    }

    #[test]
    fn test_parse_and_expression() {
        let q = parse_query("tags contain \"project\" and date before 01.01.2024").unwrap();
        assert!(matches!(q, Query::And(_, _)));
        if let Query::And(left, right) = &q {
            assert_eq!(**left, Query::TagsContain("project".to_string()));
            assert_eq!(**right, Query::DateBefore(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()));
        }
    }

    #[test]
    fn test_parse_plain_text() {
        let q = parse_query("hello world").unwrap();
        assert_eq!(q, Query::Text("hello world".to_string()));
    }

    #[test]
    fn test_parse_invalid_date() {
        let result = parse_query("date before 99.99.9999");
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_tags_contain() {
        let idx = ConcurrentIndex::new();
        let note = make_note("test", vec!["project"], vec![]);
        idx.index_note(&note);

        let results = execute_query(&idx, &Query::TagsContain("project".to_string()));
        assert_eq!(results, vec![PathBuf::from("test.md")]);
    }

    #[test]
    fn test_execute_and_query() {
        let idx = ConcurrentIndex::new();
        idx.index_note(&make_note("a", vec!["project", "todo"], vec![]));
        idx.index_note(&make_note("b", vec!["project"], vec![]));
        idx.index_note(&make_note("c", vec!["todo"], vec![]));

        let query = Query::And(
            Box::new(Query::TagsContain("project".to_string())),
            Box::new(Query::TagsContain("todo".to_string())),
        );
        let results = execute_query(&idx, &query);
        assert_eq!(results, vec![PathBuf::from("a.md")]);
    }

    #[test]
    fn test_execute_or_query() {
        let idx = ConcurrentIndex::new();
        idx.index_note(&make_note("a", vec!["project", "todo"], vec![]));
        idx.index_note(&make_note("b", vec!["project"], vec![]));
        idx.index_note(&make_note("c", vec!["todo"], vec![]));

        let query = Query::Or(
            Box::new(Query::TagsContain("project".to_string())),
            Box::new(Query::TagsContain("todo".to_string())),
        );
        let mut results = execute_query(&idx, &query);
        results.sort();
        assert_eq!(results, vec![
            PathBuf::from("a.md"),
            PathBuf::from("b.md"),
            PathBuf::from("c.md"),
        ]);
    }
}
