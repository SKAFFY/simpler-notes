use std::path::{Path, PathBuf};
use std::collections::HashMap;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use crate::note_model::ByteSpan;
use crate::parser::parse_content;
use crate::util::normalize_path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub span: ByteSpan,
    pub message: String,
    pub severity: Severity,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Severity {
    Warning,
}

pub struct Diagnostics {
    file_diagnostics: DashMap<PathBuf, Vec<Diagnostic>>,
}

impl Default for Diagnostics {
    fn default() -> Self {
        Self::new()
    }
}

impl Diagnostics {
    pub fn new() -> Self {
        Diagnostics { file_diagnostics: DashMap::new() }
    }

    /// Check one file and store results.
    pub fn check_file(
        &self,
        path: &Path,
        content: &str,
        _vault_path: &Path,
        filename_index: &HashMap<String, Vec<PathBuf>>,
    ) {
        let result = parse_content(content);
        let mut diagnostics: Vec<Diagnostic> = result.errors.into_iter()
            .map(|e| Diagnostic {
                span: ByteSpan { offset: e.span.offset, length: e.span.length },
                message: e.message,
                severity: Severity::Warning,
            })
            .collect();

        for link in &result.links {
            let raw_target = PathBuf::from(&link.file_name);
            let resolved = if raw_target.is_absolute() {
                raw_target
            } else {
                path.parent().unwrap_or(Path::new("")).join(&raw_target)
            };
            let normalized = normalize_path(&resolved);
            let link_name = normalized
                .file_stem()
                .unwrap_or(normalized.as_os_str())
                .to_string_lossy()
                .to_string();

            match filename_index.get(&link_name) {
                None => {
                    diagnostics.push(Diagnostic {
                        span: ByteSpan { offset: link.span.offset, length: link.span.length },
                        message: format!("Broken link: {} — file not found", link.file_name),
                        severity: Severity::Warning,
                    });
                }
                Some(files) if files.len() > 1 => {
                    let file_list = files.iter()
                        .map(|f| f.to_string_lossy().to_string())
                        .collect::<Vec<_>>()
                        .join(", ");
                    diagnostics.push(Diagnostic {
                        span: ByteSpan { offset: link.span.offset, length: link.span.length },
                        message: format!("Ambiguous link: {} — multiple files: {}", link_name, file_list),
                        severity: Severity::Warning,
                    });
                }
                _ => {}
            }
        }

        self.file_diagnostics.insert(path.to_path_buf(), diagnostics);
    }

    pub fn get(&self, path: &Path) -> Vec<Diagnostic> {
        self.file_diagnostics.get(path).map(|d| d.value().clone()).unwrap_or_default()
    }

    pub fn all(&self) -> Vec<(PathBuf, Vec<Diagnostic>)> {
        self.file_diagnostics.iter()
            .map(|e| (e.key().clone(), e.value().clone()))
            .collect()
    }

    pub fn remove(&self, path: &Path) {
        self.file_diagnostics.remove(path);
    }

    pub fn clear(&self) {
        self.file_diagnostics.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn empty_index() -> HashMap<String, Vec<PathBuf>> {
        HashMap::new()
    }

    #[test]
    fn test_empty_link_diagnostic() {
        let diag = Diagnostics::new();
        let vault = TempDir::new().unwrap();
        diag.check_file(&PathBuf::from("test.md"), "[[]]", vault.path(), &empty_index());
        let result = diag.get(&PathBuf::from("test.md"));
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].severity, Severity::Warning);
    }

    #[test]
    fn test_invalid_date_diagnostic() {
        let diag = Diagnostics::new();
        let vault = TempDir::new().unwrap();
        diag.check_file(&PathBuf::from("test.md"), "!32.13.2000", vault.path(), &empty_index());
        let result = diag.get(&PathBuf::from("test.md"));
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_clean_file() {
        let diag = Diagnostics::new();
        let vault = TempDir::new().unwrap();
        diag.check_file(&PathBuf::from("test.md"), "Just text", vault.path(), &empty_index());
        assert!(diag.get(&PathBuf::from("test.md")).is_empty());
    }

    #[test]
    fn test_remove_file() {
        let diag = Diagnostics::new();
        let vault = TempDir::new().unwrap();
        diag.check_file(&PathBuf::from("test.md"), "[[]]", vault.path(), &empty_index());
        diag.remove(&PathBuf::from("test.md"));
        assert!(diag.get(&PathBuf::from("test.md")).is_empty());
    }

    #[test]
    fn test_all_diagnostics() {
        let diag = Diagnostics::new();
        let vault = TempDir::new().unwrap();
        diag.check_file(&PathBuf::from("a.md"), "[[]]", vault.path(), &empty_index());
        diag.check_file(&PathBuf::from("b.md"), "!32.13.2000", vault.path(), &empty_index());
        assert_eq!(diag.all().len(), 2);
    }

    #[test]
    fn test_clear() {
        let diag = Diagnostics::new();
        let vault = TempDir::new().unwrap();
        diag.check_file(&PathBuf::from("a.md"), "[[]]", vault.path(), &empty_index());
        diag.clear();
        assert!(diag.all().is_empty());
    }

    #[test]
    fn test_broken_link() {
        let diag = Diagnostics::new();
        let vault = TempDir::new().unwrap();
        diag.check_file(&PathBuf::from("test.md"), "[[NonExistent]]", vault.path(), &empty_index());
        let result = diag.get(&PathBuf::from("test.md"));
        assert_eq!(result.len(), 1);
        assert!(result[0].message.contains("Broken link"));
    }

    #[test]
    fn test_ambiguous_link() {
        let diag = Diagnostics::new();
        let vault = TempDir::new().unwrap();
        let f1 = vault.path().join("note.md");
        let f2 = vault.path().join("sub").join("note.md");
        std::fs::create_dir_all(vault.path().join("sub")).unwrap();
        std::fs::write(&f1, "").unwrap();
        std::fs::write(&f2, "").unwrap();
        let mut index = HashMap::new();
        index.insert("note".to_string(), vec![f1, f2]);
        diag.check_file(&PathBuf::from("source.md"), "[[note]]", vault.path(), &index);
        let result = diag.get(&PathBuf::from("source.md"));
        assert_eq!(result.len(), 1);
        assert!(result[0].message.contains("Ambiguous link"));
    }

    #[test]
    fn test_valid_link_no_diagnostic() {
        let diag = Diagnostics::new();
        let vault = TempDir::new().unwrap();
        let f1 = vault.path().join("note.md");
        std::fs::write(&f1, "").unwrap();
        let mut index = HashMap::new();
        index.insert("note".to_string(), vec![f1]);
        diag.check_file(&PathBuf::from("source.md"), "[[note]]", vault.path(), &index);
        let result = diag.get(&PathBuf::from("source.md"));
        assert!(result.is_empty());
    }

    fn make_filename_index(vault_path: &Path, files: &[&str]) -> HashMap<String, Vec<PathBuf>> {
        let mut index: HashMap<String, Vec<PathBuf>> = HashMap::new();
        for f in files {
            let full = vault_path.join(f);
            if let Some(stem) = full.file_stem() {
                let name = stem.to_string_lossy().to_string();
                index.entry(name).or_default().push(full);
            }
        }
        index
    }

    #[test]
    fn test_table_driven_check_file() {
        struct Case {
            name: &'static str,
            content: &'static str,
            vault_files: &'static [&'static str],
            expected_count: usize,
            expected_messages_contain: &'static [&'static str],
        }

        let cases: Vec<Case> = vec![
            Case {
                name: "valid link resolves to single file — no diagnostic",
                content: "[[note]]",
                vault_files: &["note.md"],
                expected_count: 0,
                expected_messages_contain: &[],
            },
            Case {
                name: "broken link to non-existent file",
                content: "[[ghost]]",
                vault_files: &[],
                expected_count: 1,
                expected_messages_contain: &["Broken link"],
            },
            Case {
                name: "ambiguous link — two files with same stem",
                content: "[[note]]",
                vault_files: &["note.md", "sub/note.md"],
                expected_count: 1,
                expected_messages_contain: &["Ambiguous link"],
            },
            Case {
                name: "empty filename_index — all links broken",
                content: "[[a]] and [[b]]",
                vault_files: &[],
                expected_count: 2,
                expected_messages_contain: &["Broken link"],
            },
            Case {
                name: "multiple links with mixed validity",
                content: "[[valid]] and [[broken]] and [[also-broken]]",
                vault_files: &["valid.md"],
                expected_count: 2,
                expected_messages_contain: &["Broken link"],
            },
            Case {
                name: "no links in content — no diagnostics",
                content: "just plain text",
                vault_files: &[],
                expected_count: 0,
                expected_messages_contain: &[],
            },
            Case {
                name: "absolute path link hits is_absolute branch",
                content: "[[/tmp/some-abs-test-42]]",
                vault_files: &["existing.md"],
                expected_count: 1,
                expected_messages_contain: &["Broken link"],
            },
        ];

        for (i, case) in cases.into_iter().enumerate() {
            let vault = TempDir::new().unwrap();
            for f in case.vault_files {
                let full = vault.path().join(f);
                if let Some(parent) = full.parent() {
                    std::fs::create_dir_all(parent).unwrap();
                }
                std::fs::write(&full, "").unwrap();
            }
            let index = make_filename_index(vault.path(), case.vault_files);

            let diag = Diagnostics::new();
            let path = PathBuf::from(format!("source_{}.md", i));
            diag.check_file(&path, case.content, vault.path(), &index);

            let result = diag.get(&path);
            let mut errors: Vec<String> = Vec::new();

            if result.len() != case.expected_count {
                errors.push(format!("expected {} diagnostics, got {}", case.expected_count, result.len()));
            }

            for msg_needle in case.expected_messages_contain {
                let found = result.iter().any(|d| d.message.contains(msg_needle));
                if !found {
                    errors.push(format!("expected diagnostic containing '{}', none found", msg_needle));
                }
            }

            assert!(errors.is_empty(), "case {} ({}): {}", i, case.name, errors.join("; "));
        }
    }
}
