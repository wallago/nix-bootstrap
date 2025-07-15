use anyhow::{Context, Result, anyhow, bail};
use git2::{Repository, Status, StatusOptions};
use tempfile::{TempDir, tempdir};

pub fn get_repository_by_clone(name: &str) -> Result<(Repository, TempDir)> {
    let tmp_dir = tempdir().context("Failed to create temp directory")?;
    let config_path = tmp_dir.path().join(name);
    let repo = Repository::clone(&format!("https://github.com/wallago/{name}"), &config_path)
        .context("Failed to clone repository")?;
    if repo.is_bare() {
        bail!("Cloned repository is a bare")
    }
    if repo.is_shallow() {
        bail!("Cloned repository is a shallow")
    }
    Ok((repo, tmp_dir))
}

pub fn get_repository_by_path(path: &str) -> Result<Repository> {
    let repo = Repository::discover(path).context("Failed to discover repository")?;
    if repo.is_bare() {
        bail!("Cloned repository is a bare")
    }
    if repo.is_shallow() {
        bail!("Cloned repository is a shallow")
    }
    Ok(repo)
}

pub fn untrack_changes(repo: &Repository) -> Result<Vec<String>> {
    let mut untrack_files = Vec::new();
    let mut status_opts = StatusOptions::new();
    status_opts.include_untracked(true).include_ignored(false);
    let statuses = repo.statuses(Some(&mut status_opts))?;
    for entry in statuses.iter() {
        match entry.status() {
            Status::WT_MODIFIED
            | Status::WT_DELETED
            | Status::WT_RENAMED
            | Status::WT_TYPECHANGE
            | Status::WT_NEW => {
                let path = entry
                    .path()
                    .ok_or_else(|| anyhow!("Failed to get path of untracked file"))?;
                untrack_files.push(path.to_string());
            }
            _ => continue,
        }
    }
    Ok(untrack_files)
}
