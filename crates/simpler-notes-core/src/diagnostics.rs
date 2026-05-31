use std::path::{Path, PathBuf};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use crate::note_model::ByteSpan;
use crate::parser::parse_content;

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

impl Diagnostics {
    pub fn new() -> Self {
        Diagnostics { file_diagnostics: DashMap::new() }
    }

    /// Check one file and store results.
    pub fn check_file(&self, path: &Path, content: &str, vault_path: &Path) {
        let result = parse_content(content);
        let mut diagnostics: Vec<Diagnostic> = result.errors.into_iter()
            .map(|e| Diagnostic {
                span: ByteSpan { offset: e.span.offset, length: e.span.length },
                message: e.message,
                severity: Severity::Warning,
            })
            .collect();

        for link in &result.links {
            let full_path = vault_path.join(&link.file_name).with_extension("md");
            if !full_path.exists() {
                diagnostics.push(Diagnostic {
                    span: ByteSpan { offset: link.span.offset, length: link.span.length },
                    message: format!("Broken link: {} — file not found", link.file_name),
                    severity: Severity::Warning,
                });
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

    #[test]
    fn test_empty_link_diagnostic() {
        let diag = Diagnostics::new();
        let vault = TempDir::new().unwrap();
        diag.check_file(&PathBuf::from("test.md"), "[[]]", vault.path());
        let result = diag.get(&PathBuf::from("test.md"));
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].severity, Severity::Warning);
    }

    #[test]
    fn test_invalid_date_diagnostic() {
        let diag = Diagnostics::new();
        let vault = TempDir::new().unwrap();
        diag.check_file(&PathBuf::from("test.md"), "!32.13.2000", vault.path());
        let result = diag.get(&PathBuf::from("test.md"));
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_clean_file() {
        let diag = Diagnostics::new();
        let vault = TempDir::new().unwrap();
        diag.check_file(&PathBuf::from("test.md"), "Just text", vault.path());
        assert!(diag.get(&PathBuf::from("test.md")).is_empty());
    }

    #[test]
    fn test_remove_file() {
        let diag = Diagnostics::new();
        let vault = TempDir::new().unwrap();
        diag.check_file(&PathBuf::from("test.md"), "[[]]", vault.path());
        diag.remove(&PathBuf::from("test.md"));
        assert!(diag.get(&PathBuf::from("test.md")).is_empty());
    }

    #[test]
    fn test_all_diagnostics() {
        let diag = Diagnostics::new();
        let vault = TempDir::new().unwrap();
        diag.check_file(&PathBuf::from("a.md"), "[[]]", vault.path());
        diag.check_file(&PathBuf::from("b.md"), "!32.13.2000", vault.path());
        assert_eq!(diag.all().len(), 2);
    }

    #[test]
    fn test_clear() {
        let diag = Diagnostics::new();
        let vault = TempDir::new().unwrap();
        diag.check_file(&PathBuf::from("a.md"), "[[]]", vault.path());
        diag.clear();
        assert!(diag.all().is_empty());
    }

    #[test]
    fn test_broken_link() {
        let diag = Diagnostics::new();
        let vault = TempDir::new().unwrap();
        diag.check_file(&PathBuf::from("test.md"), "[[NonExistent]]", vault.path());
        let result = diag.get(&PathBuf::from("test.md"));
        assert_eq!(result.len(), 1);
        assert!(result[0].message.contains("Broken link"));
    }
}
