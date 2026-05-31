use std::path::Path;
use git2::{Repository, DiffOptions, StatusOptions};

pub struct GitBackend {
    repo: Repository,
}

impl GitBackend {
    pub fn open(vault_path: &Path) -> Result<Self, String> {
        let repo = Repository::open(vault_path)
            .map_err(|e| format!("Failed to open git repo: {}", e))?;
        Ok(GitBackend { repo })
    }

    pub fn init(vault_path: &Path) -> Result<Self, String> {
        let repo = Repository::init(vault_path)
            .map_err(|e| format!("Failed to init git repo: {}", e))?;
        Ok(GitBackend { repo })
    }

    pub fn is_dirty(&self) -> Result<bool, String> {
        let mut opts = StatusOptions::new();
        opts.include_untracked(true);
        let statuses = self.repo.statuses(Some(&mut opts))
            .map_err(|e| e.to_string())?;
        Ok(!statuses.is_empty())
    }

    pub fn stage_all(&self) -> Result<(), String> {
        let mut index = self.repo.index()
            .map_err(|e| e.to_string())?;
        index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)
            .map_err(|e| e.to_string())?;
        index.write().map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn commit(&self, message: &str) -> Result<String, String> {
        let signature = self.repo.signature()
            .map_err(|e| e.to_string())?;

        let oid = self.repo.index()
            .map_err(|e| e.to_string())?
            .write_tree()
            .map_err(|e| e.to_string())?;
        let tree = self.repo.find_tree(oid)
            .map_err(|e| e.to_string())?;

        let parent_commit = self.repo.head().ok()
            .and_then(|head| head.peel_to_commit().ok());

        let commit_oid = if let Some(parent) = parent_commit {
            self.repo.commit(
                Some("HEAD"),
                &signature,
                &signature,
                message,
                &tree,
                &[&parent],
            ).map_err(|e| e.to_string())?
        } else {
            self.repo.commit(
                Some("HEAD"),
                &signature,
                &signature,
                message,
                &tree,
                &[],
            ).map_err(|e| e.to_string())?
        };

        Ok(commit_oid.to_string())
    }

    pub fn get_log(&self, max_count: usize) -> Result<Vec<GitCommit>, String> {
        let mut revwalk = self.repo.revwalk()
            .map_err(|e| e.to_string())?;
        revwalk.push_head().map_err(|e| e.to_string())?;
        revwalk.set_sorting(git2::Sort::TIME).ok();

        let mut commits = Vec::new();
        for (i, oid) in revwalk.enumerate() {
            if i >= max_count {
                break;
            }
            let oid = oid.map_err(|e| e.to_string())?;
            let commit = self.repo.find_commit(oid)
                .map_err(|e| e.to_string())?;
            commits.push(GitCommit {
                oid: oid.to_string(),
                message: commit.message().unwrap_or("").to_string(),
                author: commit.author().name().unwrap_or("").to_string(),
                time: commit.time().seconds(),
            });
        }
        Ok(commits)
    }

    pub fn push(&self) -> Result<(), String> {
        let mut remote = self.repo.find_remote("origin")
            .map_err(|e| format!("No remote 'origin': {}", e))?;
        remote.push(&["refs/heads/master"], None)
            .map_err(|e| format!("Push failed: {}", e))?;
        Ok(())
    }

    pub fn pull(&self) -> Result<(), String> {
        let mut remote = self.repo.find_remote("origin")
            .map_err(|e| format!("No remote 'origin': {}", e))?;
        remote.fetch(&["refs/heads/master"], None, None)
            .map_err(|e| format!("Fetch failed: {}", e))?;

        let fetch_head = self.repo.find_reference("FETCH_HEAD")
            .map_err(|e| e.to_string())?;
        let fetch_commit = fetch_head.peel_to_commit()
            .map_err(|e| e.to_string())?;

        let head = self.repo.head()
            .map_err(|e| e.to_string())?;
        let head_commit = head.peel_to_commit()
            .map_err(|e| e.to_string())?;

        if head_commit.id() == fetch_commit.id() {
            return Ok(());
        }

        let annotated_fetch = self.repo.find_annotated_commit(fetch_commit.id())
            .map_err(|e| e.to_string())?;
        let mut merge_opts = git2::MergeOptions::new();
        self.repo.merge(&[&annotated_fetch], Some(&mut merge_opts), None)
            .map_err(|e| format!("Merge failed: {}", e))?;

        if self.repo.index().map_err(|e| e.to_string())?.has_conflicts() {
            return Err("Merge conflicts detected after pull".to_string());
        }

        let signature = self.repo.signature().map_err(|e| e.to_string())?;
        let tree = self.repo.find_tree(
            self.repo.index().map_err(|e| e.to_string())?.write_tree().map_err(|e| e.to_string())?
        ).map_err(|e| e.to_string())?;

        self.repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            "Merge",
            &tree,
            &[&head_commit, &fetch_commit],
        ).map_err(|e| e.to_string())?;

        Ok(())
    }

    pub fn unpushed_count(&self) -> Result<usize, String> {
        let head = self.repo.head().map_err(|e| e.to_string())?;
        let head_oid = head.peel_to_commit().map_err(|e| e.to_string())?.id();

        let upstream = self.repo.find_reference("refs/remotes/origin/master");
        match upstream {
            Ok(up) => {
                let up_oid = up.peel_to_commit().map_err(|e| e.to_string())?.id();
                let mut revwalk = self.repo.revwalk().map_err(|e| e.to_string())?;
                revwalk.push(head_oid).map_err(|e| e.to_string())?;
                revwalk.hide(up_oid).map_err(|e| e.to_string())?;
                Ok(revwalk.count())
            }
            Err(_) => {
                let mut revwalk = self.repo.revwalk().map_err(|e| e.to_string())?;
                revwalk.push_head().map_err(|e| e.to_string())?;
                Ok(revwalk.count())
            }
        }
    }

    pub fn close(&self) {
    }

    pub fn diff_unstaged(&self) -> Result<String, String> {
        let diff = self.repo.diff_index_to_workdir(
            None,
            Some(DiffOptions::new().show_untracked_content(true)),
        ).map_err(|e| e.to_string())?;

        let mut result = String::new();
        diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
            if let Ok(content) = std::str::from_utf8(line.content()) {
                let prefix = match line.origin() {
                    '+' => "+",
                    '-' => "-",
                    _ => " ",
                };
                result.push_str(&format!("{}{}", prefix, content));
            }
            true
        }).map_err(|e| e.to_string())?;

        Ok(result)
    }
}

pub struct GitCommit {
    pub oid: String,
    pub message: String,
    pub author: String,
    pub time: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_init_and_is_dirty() {
        let dir = TempDir::new().unwrap();
        let git = GitBackend::init(dir.path()).unwrap();
        assert!(!git.is_dirty().unwrap());

        fs::write(dir.path().join("test.md"), "hello").unwrap();
        assert!(git.is_dirty().unwrap());
    }

    #[test]
    fn test_commit() {
        let dir = TempDir::new().unwrap();
        let git = GitBackend::init(dir.path()).unwrap();
        fs::write(dir.path().join("test.md"), "hello").unwrap();
        git.stage_all().unwrap();
        let oid = git.commit("Initial commit").unwrap();
        assert!(!oid.is_empty());
    }

    #[test]
    fn test_get_log() {
        let dir = TempDir::new().unwrap();
        let git = GitBackend::init(dir.path()).unwrap();
        fs::write(dir.path().join("test.md"), "hello").unwrap();
        git.stage_all().unwrap();
        git.commit("First").unwrap();

        let log = git.get_log(10).unwrap();
        assert_eq!(log.len(), 1);
        assert_eq!(log[0].message, "First");
    }

    #[test]
    fn test_open_not_a_repo() {
        let dir = TempDir::new().unwrap();
        let result = GitBackend::open(dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_diff_unstaged() {
        let dir = TempDir::new().unwrap();
        let git = GitBackend::init(dir.path()).unwrap();
        fs::write(dir.path().join("test.md"), "line1\nline2\n").unwrap();
        git.stage_all().unwrap();
        git.commit("Initial").unwrap();

        fs::write(dir.path().join("test.md"), "line1\nmodified\n").unwrap();
        let diff = git.diff_unstaged().unwrap();
        assert!(diff.contains("-line2"));
        assert!(diff.contains("+modified"));
    }
}
