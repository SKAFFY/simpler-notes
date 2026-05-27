use std::path::Path;

#[derive(Debug)]
pub struct GitBackend {
    repo_path: std::path::PathBuf,
}

impl GitBackend {
    pub fn open(path: &Path) -> Result<Self, String> {
        if !path.join(".git").exists() {
            return Err("Not a git repository".to_string());
        }
        Ok(Self { repo_path: path.to_path_buf() })
    }

    pub fn auto_commit(&self, message: &str) -> Result<(), String> {
        let repo = git2::Repository::open(&self.repo_path)
            .map_err(|e| format!("Failed to open repo: {}", e))?;

        let mut index = repo.index()
            .map_err(|e| format!("Failed to open index: {}", e))?;

        index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)
            .map_err(|e| format!("Failed to add files: {}", e))?;

        index.write()
            .map_err(|e| format!("Failed to write index: {}", e))?;

        let tree_oid = index.write_tree()
            .map_err(|e| format!("Failed to write tree: {}", e))?;

        let tree = repo.find_tree(tree_oid)
            .map_err(|e| format!("Failed to find tree: {}", e))?;

        let author = git2::Signature::now("Simpler Notes", "notes@simpler.app")
            .map_err(|e| format!("Failed to create signature: {}", e))?;

        let parent = repo.head().ok()
            .and_then(|head| head.target())
            .and_then(|oid| repo.find_commit(oid).ok());

        let _commit = match parent {
            Some(parent) => repo.commit(
                Some("HEAD"),
                &author,
                &author,
                message,
                &tree,
                &[&parent],
            ),
            None => repo.commit(
                Some("HEAD"),
                &author,
                &author,
                message,
                &tree,
                &[] as &[&git2::Commit],
            ),
        }.map_err(|e| format!("Failed to commit: {}", e))?;

        Ok(())
    }

    pub fn push(&self) -> Result<(), String> {
        let repo = git2::Repository::open(&self.repo_path)
            .map_err(|e| format!("Failed to open repo: {}", e))?;

        let mut remote = repo.find_remote("origin")
            .map_err(|_| "No remote 'origin' configured".to_string())?;

        remote.push(&["refs/heads/main"], None)
            .map_err(|e| format!("Failed to push: {}", e))?;

        Ok(())
    }

    pub fn pull(&self) -> Result<(), String> {
        let repo = git2::Repository::open(&self.repo_path)
            .map_err(|e| format!("Failed to open repo: {}", e))?;

        let mut remote = repo.find_remote("origin")
            .map_err(|_| "No remote 'origin' configured".to_string())?;

        remote.fetch(&["main"], None, None)
            .map_err(|e| format!("Failed to fetch: {}", e))?;

        let fetch_head = repo.find_reference("FETCH_HEAD")
            .map_err(|e| format!("Failed to find FETCH_HEAD: {}", e))?;
        let fetch_commit_oid = fetch_head.target()
            .ok_or("No target in FETCH_HEAD".to_string())?;
        let fetch_commit = repo.find_annotated_commit(fetch_commit_oid)
            .map_err(|e| format!("Failed to find annotated commit: {}", e))?;

        let analysis = repo.merge_analysis(&[&fetch_commit])
            .map_err(|e| format!("Failed to merge analysis: {}", e))?;

        if analysis.0.is_up_to_date() {
            return Ok(());
        }

        let refname = "refs/heads/main";
        let branch_ref = repo.find_reference(refname)
            .map_err(|e| format!("Failed to find branch ref: {}", e))?;
        let branch_oid = branch_ref.target()
            .ok_or("No target in branch ref".to_string())?;
        let branch_commit = repo.find_commit(branch_oid)
            .map_err(|e| format!("Failed to find branch commit: {}", e))?;

        repo.merge(&[&fetch_commit], None, None)
            .map_err(|e| format!("Failed to merge: {}", e))?;

        if repo.index().map(|i| i.has_conflicts()).unwrap_or(false) {
            return Err("Merge conflicts detected. Resolve manually.".to_string());
        }

        let tree_oid = repo.index()
            .map_err(|e| format!("Failed to open index: {}", e))?
            .write_tree()
            .map_err(|e| format!("Failed to write merge tree: {}", e))?;

        let tree = repo.find_tree(tree_oid)
            .map_err(|e| format!("Failed to find merge tree: {}", e))?;

        let sig = git2::Signature::now("Simpler Notes", "notes@simpler.app")
            .map_err(|e| format!("Failed to create signature: {}", e))?;

        let fetch_commit_obj = repo.find_commit(fetch_commit_oid)
            .map_err(|e| format!("Failed to find fetch commit: {}", e))?;

        repo.commit(
            Some(refname),
            &sig,
            &sig,
            "Merge remote-tracking branch 'origin/main'",
            &tree,
            &[&branch_commit, &fetch_commit_obj],
        ).map_err(|e| format!("Failed to commit merge: {}", e))?;

        Ok(())
    }

    pub fn status(&self) -> Result<String, String> {
        let repo = git2::Repository::open(&self.repo_path)
            .map_err(|e| format!("Failed to open repo: {}", e))?;

        let statuses = repo.statuses(None)
            .map_err(|e| format!("Failed to get statuses: {}", e))?;

        let clean = statuses.iter().all(|s| s.status() == git2::Status::CURRENT);

        Ok(if clean {
            "clean".to_string()
        } else {
            format!("{} dirty file(s)", statuses.len())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_git_open_non_repo() {
        let dir = std::env::temp_dir().join("simpler_notes_git_test_non_repo");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let result = GitBackend::open(&dir);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Not a git repository"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_git_init_and_auto_commit() {
        let dir = std::env::temp_dir().join("simpler_notes_git_test_commit");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        {
            let repo = git2::Repository::init(&dir).unwrap();
            let sig = git2::Signature::now("test", "test@test.com").unwrap();
            let tree_oid = {
                let mut idx = repo.index().unwrap();
                idx.write().unwrap();
                idx.write_tree().unwrap()
            };
            let tree = repo.find_tree(tree_oid).unwrap();
            repo.commit(Some("HEAD"), &sig, &sig, "initial", &tree, &[]).unwrap();
        }

        let git = GitBackend::open(&dir).unwrap();

        fs::write(dir.join("test.md"), "# Hello").unwrap();

        let result = git.auto_commit("Add test.md");
        assert!(result.is_ok());

        let status = git.status().unwrap();
        assert_eq!(status, "clean");

        let _ = fs::remove_dir_all(&dir);
    }
}
