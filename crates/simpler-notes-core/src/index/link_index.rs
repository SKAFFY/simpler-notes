use std::path::{Path, PathBuf};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use crate::note_model::ByteSpan;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkEntry {
    pub source: PathBuf,
    pub target: PathBuf,
    pub label: String,
    pub span: ByteSpan,
}

pub struct LinkIndex {
    by_target: DashMap<PathBuf, Vec<LinkEntry>>,
    by_source: DashMap<PathBuf, Vec<LinkEntry>>,
}

impl Default for LinkIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl LinkIndex {
    pub fn new() -> Self {
        LinkIndex {
            by_target: DashMap::new(),
            by_source: DashMap::new(),
        }
    }

    pub fn add(&self, _source: PathBuf, entry: LinkEntry) {
        self.by_target.entry(entry.target.clone()).or_default().push(entry.clone());
        self.by_source.entry(entry.source.clone()).or_default().push(entry);
    }

    pub fn remove_file(&self, path: &Path) {
        if let Some((_, entries)) = self.by_source.remove(path) {
            for e in &entries {
                if let Some(mut vec) = self.by_target.get_mut(&e.target) {
                    vec.retain(|v| v.source != path);
                    if vec.is_empty() {
                        drop(vec);
                        self.by_target.remove(&e.target);
                    }
                }
            }
        }
    }

    pub fn backlinks(&self, target: &Path) -> Vec<LinkEntry> {
        self.by_target.get(target).map(|e| e.value().clone()).unwrap_or_default()
    }

    pub fn outgoing(&self, source: &Path) -> Vec<LinkEntry> {
        self.by_source.get(source).map(|e| e.value().clone()).unwrap_or_default()
    }

    pub fn update_target(&self, old: &Path, new: &Path) {
        if let Some((_, entries)) = self.by_target.remove(old) {
            for mut entry in entries {
                entry.target = new.to_path_buf();
                self.by_target.entry(new.to_path_buf()).or_default().push(entry.clone());
                if let Some(mut vec) = self.by_source.get_mut(&entry.source) {
                    for e in vec.iter_mut() {
                        if e.target == old {
                            e.target = new.to_path_buf();
                        }
                    }
                }
            }
        }
    }

    pub fn clear(&self) {
        self.by_target.clear();
        self.by_source.clear();
    }

    pub fn all_targets(&self) -> Vec<PathBuf> {
        let mut targets: Vec<PathBuf> = self.by_target.iter()
            .map(|e| e.key().clone())
            .collect();
        targets.sort();
        targets.dedup();
        targets
    }

    pub fn iter(&self) -> Vec<LinkEntry> {
        let mut result = Vec::new();
        for e in self.by_source.iter() {
            result.extend(e.value().iter().cloned());
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entry(source: &str, target: &str, label: &str) -> LinkEntry {
        LinkEntry {
            source: PathBuf::from(source),
            target: PathBuf::from(target),
            label: label.to_string(),
            span: ByteSpan { offset: 0, length: 10 },
        }
    }

    #[test]
    fn test_add_and_backlinks() {
        let index = LinkIndex::new();
        let source = PathBuf::from("a.md");
        let target = PathBuf::from("b.md");
        let entry = make_entry("a.md", "b.md", "B");
        index.add(source.clone(), entry);
        let backlinks = index.backlinks(&target);
        assert_eq!(backlinks.len(), 1);
        assert_eq!(backlinks[0].source, source);
    }

    #[test]
    fn test_outgoing() {
        let index = LinkIndex::new();
        let source = PathBuf::from("a.md");
        let target = PathBuf::from("b.md");
        let entry = make_entry("a.md", "b.md", "B");
        index.add(source.clone(), entry);
        let outgoing = index.outgoing(&source);
        assert_eq!(outgoing.len(), 1);
        assert_eq!(outgoing[0].target, target);
    }

    #[test]
    fn test_remove_file() {
        let index = LinkIndex::new();
        let source = PathBuf::from("a.md");
        let target = PathBuf::from("b.md");
        let entry = make_entry("a.md", "b.md", "B");
        index.add(source.clone(), entry);
        index.remove_file(&source);
        assert!(index.backlinks(&target).is_empty());
    }

    #[test]
    fn test_all_targets() {
        let index = LinkIndex::new();
        let entry_a = make_entry("x.md", "t.md", "T");
        let entry_b = make_entry("y.md", "t.md", "T");
        index.add(PathBuf::from("x.md"), entry_a);
        index.add(PathBuf::from("y.md"), entry_b);
        let targets = index.all_targets();
        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0], PathBuf::from("t.md"));
    }

    #[test]
    fn test_clear() {
        let index = LinkIndex::new();
        index.add(PathBuf::from("a.md"), make_entry("a.md", "b.md", "B"));
        index.clear();
        assert!(index.backlinks(&PathBuf::from("b.md")).is_empty());
    }

    #[test]
    fn test_update_target() {
        let index = LinkIndex::new();
        let old_target = PathBuf::from("old.md");
        let new_target = PathBuf::from("new.md");
        index.add(PathBuf::from("a.md"), make_entry("a.md", "old.md", "link"));
        // backlinks before update
        assert_eq!(index.backlinks(&old_target).len(), 1);
        assert!(index.backlinks(&new_target).is_empty());
        // update
        index.update_target(&old_target, &new_target);
        // backlinks after update
        assert!(index.backlinks(&old_target).is_empty());
        assert_eq!(index.backlinks(&new_target).len(), 1);
        assert_eq!(index.backlinks(&new_target)[0].target, new_target);
    }

    #[test]
    fn test_outgoing_is_o1() {
        let index = LinkIndex::new();
        index.add(PathBuf::from("a.md"), make_entry("a.md", "b.md", "B"));
        index.add(PathBuf::from("a.md"), make_entry("a.md", "c.md", "C"));
        let outgoing = index.outgoing(&PathBuf::from("a.md"));
        assert_eq!(outgoing.len(), 2);
        assert!(outgoing.iter().any(|e| e.target == PathBuf::from("b.md")));
        assert!(outgoing.iter().any(|e| e.target == PathBuf::from("c.md")));
        // file with no outgoing
        assert!(index.outgoing(&PathBuf::from("orphan.md")).is_empty());
    }

    #[test]
    fn test_remove_file_cleans_both_maps() {
        let index = LinkIndex::new();
        index.add(PathBuf::from("a.md"), make_entry("a.md", "b.md", "B"));
        index.add(PathBuf::from("a.md"), make_entry("a.md", "c.md", "C"));
        index.remove_file(&PathBuf::from("a.md"));
        assert!(index.outgoing(&PathBuf::from("a.md")).is_empty());
        assert!(index.backlinks(&PathBuf::from("b.md")).is_empty());
        assert!(index.backlinks(&PathBuf::from("c.md")).is_empty());
    }

    #[test]
    fn test_add_maintains_both_maps() {
        let index = LinkIndex::new();
        let entry = make_entry("a.md", "b.md", "B");
        index.add(PathBuf::from("a.md"), entry);
        // by_source
        let outgoing = index.outgoing(&PathBuf::from("a.md"));
        assert_eq!(outgoing.len(), 1);
        assert_eq!(outgoing[0].target, PathBuf::from("b.md"));
        // by_target
        let backlinks = index.backlinks(&PathBuf::from("b.md"));
        assert_eq!(backlinks.len(), 1);
        assert_eq!(backlinks[0].source, PathBuf::from("a.md"));
    }

    #[test]
    fn test_table_driven_link_index() {
        struct Case {
            name: &'static str,
            setup: fn(&LinkIndex),
            check: fn(&LinkIndex, &mut Vec<String>),
        }

        let cases: Vec<Case> = vec![
            Case {
                name: "no backlinks for unknown target",
                setup: |_| {},
                check: |idx, errors| {
                    let bl = idx.backlinks(&PathBuf::from("ghost.md"));
                    if bl.len() != 0 { errors.push(format!("expected 0 backlinks, got {}", bl.len())); }
                },
            },
            Case {
                name: "multiple sources link to same target",
                setup: |idx| {
                    idx.add(PathBuf::from("a.md"), make_entry("a.md", "shared.md", "shared"));
                    idx.add(PathBuf::from("b.md"), make_entry("b.md", "shared.md", "shared"));
                    idx.add(PathBuf::from("c.md"), make_entry("c.md", "shared.md", "shared"));
                },
                check: |idx, errors| {
                    let bl = idx.backlinks(&PathBuf::from("shared.md"));
                    if bl.len() != 3 { errors.push(format!("expected 3 backlinks, got {}", bl.len())); }
                },
            },
            Case {
                name: "circular link a->b->a",
                setup: |idx| {
                    idx.add(PathBuf::from("a.md"), make_entry("a.md", "b.md", "b"));
                    idx.add(PathBuf::from("b.md"), make_entry("b.md", "a.md", "a"));
                },
                check: |idx, errors| {
                    let bl_a = idx.backlinks(&PathBuf::from("a.md"));
                    let bl_b = idx.backlinks(&PathBuf::from("b.md"));
                    if bl_a.len() != 1 { errors.push(format!("expected 1 backlink to a, got {}", bl_a.len())); }
                    if bl_b.len() != 1 { errors.push(format!("expected 1 backlink to b, got {}", bl_b.len())); }
                    if bl_a[0].source != PathBuf::from("b.md") { errors.push("a backlink source wrong".into()); }
                    if bl_b[0].source != PathBuf::from("a.md") { errors.push("b backlink source wrong".into()); }
                },
            },
            Case {
                name: "outgoing is empty for file with no links",
                setup: |_| {},
                check: |idx, errors| {
                    let og = idx.outgoing(&PathBuf::from("orphan.md"));
                    if og.len() != 0 { errors.push(format!("expected 0 outgoing, got {}", og.len())); }
                },
            },
            Case {
                name: "remove_file with no entries is a no-op",
                setup: |_| {},
                check: |idx, errors| {
                    idx.remove_file(&PathBuf::from("nonexistent.md"));
                    if idx.all_targets().len() != 0 { errors.push("all_targets should be empty after no-op remove".into()); }
                },
            },
            Case {
                name: "update replaces entries for a source",
                setup: |idx| {
                    idx.add(PathBuf::from("a.md"), make_entry("a.md", "old.md", "old"));
                },
                check: |idx, errors| {
                    idx.remove_file(&PathBuf::from("a.md"));
                    idx.add(PathBuf::from("a.md"), make_entry("a.md", "new.md", "new"));
                    let bl_old = idx.backlinks(&PathBuf::from("old.md"));
                    let bl_new = idx.backlinks(&PathBuf::from("new.md"));
                    if bl_old.len() != 0 { errors.push("old backlink should be gone after update".into()); }
                    if bl_new.len() != 1 { errors.push("new backlink should exist after update".into()); }
                },
            },
        ];

        for (i, case) in cases.into_iter().enumerate() {
            let index = LinkIndex::new();
            (case.setup)(&index);
            let mut errors: Vec<String> = Vec::new();
            (case.check)(&index, &mut errors);
            assert!(errors.is_empty(), "case {} ({}): {}", i, case.name, errors.join("; "));
        }
    }
}
