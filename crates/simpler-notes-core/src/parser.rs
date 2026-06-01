use regex::Regex;
use chrono::NaiveDate;
use crate::note_model::ByteSpan;

#[derive(Debug, Clone, PartialEq)]
pub struct LinkSpan {
    pub file_name: String,
    pub label: String,
    pub span: ByteSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TagSpan {
    pub name: String,
    pub span: ByteSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DateSpan {
    pub date: NaiveDate,
    pub raw: String,
    pub span: ByteSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    pub span: ByteSpan,
    pub message: String,
    pub kind: ParseErrorKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParseErrorKind {
    InvalidDate,
    EmptyLink,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParseResult {
    pub links: Vec<LinkSpan>,
    pub tags: Vec<TagSpan>,
    pub dates: Vec<DateSpan>,
    pub errors: Vec<ParseError>,
}

/// Parse markdown content extracting [[wiki-links]], @tags, and !dates.
///
/// Currently uses three separate regex passes.
/// Future: single-pass parser for performance.
pub fn parse_content(text: &str) -> ParseResult {
    let mut errors = Vec::new();
    let links = parse_links(text, &mut errors);
    let tags = parse_tags(text);
    let dates = parse_dates(text, &mut errors);
    ParseResult { links, tags, dates, errors }
}

fn parse_links(text: &str, errors: &mut Vec<ParseError>) -> Vec<LinkSpan> {
    let re = Regex::new(r"\[\[(.*?)(?:\|(.*?))?\]\]").unwrap();
    let mut links = Vec::new();

    for cap in re.captures_iter(text) {
        let m = cap.get(0).unwrap();
        let span = ByteSpan { offset: m.start(), length: m.end() - m.start() };
        let inner = cap.get(1).map_or("", |m| m.as_str());
        let label = cap.get(2).map_or(inner, |m| m.as_str());

        if inner.is_empty() {
            errors.push(ParseError {
                span,
                message: "Empty link".to_string(),
                kind: ParseErrorKind::EmptyLink,
            });
            continue;
        }

        links.push(LinkSpan {
            file_name: inner.to_string(),
            label: label.to_string(),
            span,
        });
    }
    links
}

fn parse_tags(text: &str) -> Vec<TagSpan> {
    let re = Regex::new(r"(?m:^|\s)@([a-zA-Zа-яА-Я0-9_\-]+)").unwrap();
    let mut tags = Vec::new();

    for cap in re.captures_iter(text) {
        let name = cap.get(1).unwrap().as_str().to_string();
        let m = cap.get(0).unwrap();
        let leading_len = m.as_str().chars().take_while(|&c| c == ' ' || c == '\n').count();
        let offset = m.start() + leading_len;
        let length = name.len() + 1;

        tags.push(TagSpan { name, span: ByteSpan { offset, length } });
    }
    tags
}

fn parse_dates(text: &str, errors: &mut Vec<ParseError>) -> Vec<DateSpan> {
    let re = Regex::new(r"(?m:^|\s)!(\d{2})\.(\d{2})\.(\d{4})\b").unwrap();
    let mut dates = Vec::new();

    for cap in re.captures_iter(text) {
        let m = cap.get(0).unwrap();
        let span = ByteSpan { offset: m.start(), length: m.end() - m.start() };
        let raw = format!("!{}.{}.{}", &cap[1], &cap[2], &cap[3]);
        let day: u32 = cap[1].parse().unwrap();
        let month: u32 = cap[2].parse().unwrap();
        let year: i32 = cap[3].parse().unwrap();

        match NaiveDate::from_ymd_opt(year, month, day) {
            Some(date) => dates.push(DateSpan { date, raw, span }),
            None => errors.push(ParseError {
                span,
                message: format!("Invalid date: {}.{}.{}", day, month, year),
                kind: ParseErrorKind::InvalidDate,
            }),
        }
    }
    dates
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_simple_link() {
        let result = parse_content("Hello [[Note Name]] world");
        assert_eq!(result.links.len(), 1);
        assert_eq!(result.links[0].file_name, "Note Name");
        assert_eq!(result.links[0].label, "Note Name");
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_link_with_alias() {
        let result = parse_content("[[Note Name|Display Label]]");
        assert_eq!(result.links.len(), 1);
        assert_eq!(result.links[0].file_name, "Note Name");
        assert_eq!(result.links[0].label, "Display Label");
    }

    #[test]
    fn test_empty_link_is_error() {
        let result = parse_content("[[]]");
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0].kind, ParseErrorKind::EmptyLink);
    }

    #[test]
    fn test_tags() {
        let result = parse_content("@project @todo @project");
        assert_eq!(result.tags.len(), 3);
        assert_eq!(result.tags[0].name, "project");
        assert_eq!(result.tags[1].name, "todo");
    }

    #[test]
    fn test_tags_not_at_email() {
        let result = parse_content("user@host.com");
        assert!(result.tags.is_empty());
    }

    #[test]
    fn test_double_at_not_tag() {
        let result = parse_content("@@notag");
        assert!(result.tags.is_empty());
    }

    #[test]
    fn test_valid_date() {
        let result = parse_content("Meeting !21.07.2003");
        assert_eq!(result.dates.len(), 1);
        assert_eq!(result.dates[0].raw, "!21.07.2003");
        assert_eq!(result.dates[0].date, NaiveDate::from_ymd_opt(2003, 7, 21).unwrap());
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_invalid_date() {
        let result = parse_content("!32.13.2000");
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0].kind, ParseErrorKind::InvalidDate);
    }

    #[test]
    fn test_empty_text() {
        let result = parse_content("");
        assert!(result.links.is_empty());
        assert!(result.tags.is_empty());
        assert!(result.dates.is_empty());
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_no_tags_or_dates() {
        let result = parse_content("Just plain text");
        assert!(result.links.is_empty());
        assert!(result.tags.is_empty());
        assert!(result.dates.is_empty());
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_multiline() {
        let text = "# Header\n\n@tag1\n\n[[Link]]\n\n!21.07.2003";
        let result = parse_content(text);
        assert_eq!(result.tags.len(), 1);
        assert_eq!(result.links.len(), 1);
        assert_eq!(result.dates.len(), 1);
    }

    #[test]
    fn test_broken_link_diagnostics() {
        let result = parse_content("[[Valid]] and [[]]");
        assert_eq!(result.links.len(), 1);
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0].kind, ParseErrorKind::EmptyLink);
    }

    #[test]
    fn test_table_driven_links() {
        struct Case {
            input: &'static str,
            expected_links: usize,
            expected_errors: usize,
            first_file_name: Option<&'static str>,
            first_label: Option<&'static str>,
        }

        let cases = vec![
            Case { input: "[[simple]]", expected_links: 1, expected_errors: 0, first_file_name: Some("simple"), first_label: Some("simple") },
            Case { input: "[[with spaces]]", expected_links: 1, expected_errors: 0, first_file_name: Some("with spaces"), first_label: Some("with spaces") },
            Case { input: "[[a|b]]", expected_links: 1, expected_errors: 0, first_file_name: Some("a"), first_label: Some("b") },
            Case { input: "[[|empty label]]", expected_links: 0, expected_errors: 1, first_file_name: None, first_label: None },
            Case { input: "[[multi|line|pipe]]", expected_links: 1, expected_errors: 0, first_file_name: Some("multi"), first_label: Some("line|pipe") },
            Case { input: "no brackets", expected_links: 0, expected_errors: 0, first_file_name: None, first_label: None },
            Case { input: "[[incomplete", expected_links: 0, expected_errors: 0, first_file_name: None, first_label: None },
            Case { input: "incomplete]]", expected_links: 0, expected_errors: 0, first_file_name: None, first_label: None },
            Case { input: "[[double]] [[links]]", expected_links: 2, expected_errors: 0, first_file_name: Some("double"), first_label: Some("double") },
        ];

        for (i, case) in cases.into_iter().enumerate() {
            let result = parse_content(case.input);
            assert_eq!(result.links.len(), case.expected_links, "case {}: links count mismatch", i);
            assert_eq!(result.errors.len(), case.expected_errors, "case {}: errors count mismatch", i);
            if let (Some(fname), Some(flabel)) = (case.first_file_name, case.first_label) {
                assert_eq!(result.links[0].file_name, fname, "case {}: file_name mismatch", i);
                assert_eq!(result.links[0].label, flabel, "case {}: label mismatch", i);
            }
        }
    }

    #[test]
    fn test_table_driven_dates() {
        struct Case {
            input: &'static str,
            expected_dates: usize,
            expected_errors: usize,
        }

        let cases = vec![
            Case { input: "!01.01.2024", expected_dates: 1, expected_errors: 0 },
            Case { input: "!31.12.1999", expected_dates: 1, expected_errors: 0 },
            Case { input: "!29.02.2020", expected_dates: 1, expected_errors: 0 }, // leap year
            Case { input: "!29.02.2021", expected_dates: 0, expected_errors: 1 }, // not leap year
            Case { input: "!00.01.2024", expected_dates: 0, expected_errors: 1 }, // day 0
            Case { input: "!01.00.2024", expected_dates: 0, expected_errors: 1 }, // month 0
            Case { input: "text no date", expected_dates: 0, expected_errors: 0 },
            Case { input: "!!01.01.2024", expected_dates: 0, expected_errors: 0 },
        ];

        for (i, case) in cases.into_iter().enumerate() {
            let result = parse_content(case.input);
            assert_eq!(result.dates.len(), case.expected_dates, "case {}: dates count mismatch", i);
            assert_eq!(result.errors.len(), case.expected_errors, "case {}: errors count mismatch", i);
        }
    }

    #[test]
    fn test_table_driven_tags() {
        struct Case {
            input: &'static str,
            expected_tags: Vec<&'static str>,
        }

        let cases = vec![
            Case { input: "@tag", expected_tags: vec!["tag"] },
            Case { input: "@tag1 @tag2", expected_tags: vec!["tag1", "tag2"] },
            Case { input: "@tag with spaces", expected_tags: vec!["tag"] },
            Case { input: "no tags here", expected_tags: vec![] },
            Case { input: "@TAG123", expected_tags: vec!["TAG123"] },
            Case { input: "@under_score", expected_tags: vec!["under_score"] },
        ];

        for (i, case) in cases.into_iter().enumerate() {
            let result = parse_content(case.input);
            let names: Vec<&str> = result.tags.iter().map(|t| t.name.as_str()).collect();
            assert_eq!(names, case.expected_tags, "case {}: tags mismatch", i);
        }
    }
}
