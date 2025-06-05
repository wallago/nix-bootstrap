use anyhow::{Context, Result, bail};
use git2::Repository;
use tempfile::{TempDir, tempdir};

pub fn git_clone_repository(name: &str) -> Result<Repository> {
    let tmp_dir = tempdir().context("Failed to create temp directory")?;
    tracing::info!(
        "📂 Temporary directory created at: {}",
        tmp_dir.path().display()
    );

    let config_path = tmp_dir.path().join(name);
    let repo = Repository::clone(&format!("https://github.com/wallago/{name}"), &config_path)?;
    if repo.is_bare() {
        bail!("Cloned repository is a bare")
    }
    if repo.is_shallow() {
        bail!("Cloned repository is a shallow")
    }
    Ok(repo)
}
