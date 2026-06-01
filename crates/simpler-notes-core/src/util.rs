use std::path::{Path, PathBuf, Component};

/// Normalize a path by resolving `.` and `..` components without I/O.
pub fn normalize_path(path: &Path) -> PathBuf {
    let mut components = Vec::new();
    for component in path.components() {
        match component {
            Component::CurDir => continue,
            Component::ParentDir => {
                components.pop();
            }
            other => components.push(other),
        }
    }
    components.iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestCase {
        input: &'static str,
        expected: &'static str,
    }

    #[test]
    fn test_normalize_path() {
        let cases = vec![
            TestCase { input: "a/b/c", expected: "a/b/c" },
            TestCase { input: "a/../b", expected: "b" },
            TestCase { input: "./a/b", expected: "a/b" },
            TestCase { input: "", expected: "" },
            TestCase { input: "a/./b", expected: "a/b" },
            TestCase { input: "a/b/../..", expected: "" },
            TestCase { input: "../a", expected: "a" },
        ];
        for c in cases {
            assert_eq!(
                normalize_path(Path::new(c.input)),
                PathBuf::from(c.expected),
                "normalize_path({:?})", c.input
            );
        }
    }
}
