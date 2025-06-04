use std::path::PathBuf;

use anyhow::Result;
use git2::{Repository, Status, StatusOptions};
use tempfile::TempDir;

use crate::config::Config;

pub fn initialize_nix_starter_config(tmp_dir: &TempDir) -> Result<PathBuf> {
    let config_path = tmp_dir.path().join("nix-starter-config");
    let config_repo = git2::Repository::clone(
        "https://github.com/wallago/nix-starter-config",
        &config_path,
    )?;
    if config_repo.is_bare() {
        anyhow::bail!("Cloned repository is a bare")
    }
    if config_repo.is_shallow() {
        anyhow::bail!("Cloned repository is a shallow")
    }
    tracing::info!("Config path at {}", config_path.display());
    Ok(config_path)
}

pub fn initialize_nix_config(tmp_dir: &TempDir) -> Result<PathBuf> {
    let config_path = tmp_dir.path().join("nix-config");
    let config_repo =
        git2::Repository::clone("https://github.com/wallago/nix-config", &config_path)?;
    if config_repo.is_bare() {
        anyhow::bail!("Cloned repository is a bare")
    }
    if config_repo.is_shallow() {
        anyhow::bail!("Cloned repository is a shallow")
    }
    tracing::info!("Config path at {}", config_path.display());
    Ok(config_path)
}

pub fn untracked_changes(config: &Config) -> Result<()> {
    let repo = Repository::discover(config.path.clone().unwrap())?;
    let mut status_opts = StatusOptions::new();
    status_opts.include_untracked(true).include_ignored(false);
    let statuses = repo.statuses(Some(&mut status_opts))?;
    tracing::info!("Git repository unstaged changes:");
    for entry in statuses.iter() {
        let s = entry.status();
        let file = entry.path().unwrap_or("<unknown>");
        if s.contains(Status::WT_MODIFIED)
            || s.contains(Status::WT_DELETED)
            || s.contains(Status::WT_RENAMED)
            || s.contains(Status::WT_TYPECHANGE)
        {
            tracing::info!("  modified: {file}");
        } else if s.contains(Status::WT_NEW) {
            tracing::info!("  untracked: {file}");
        }
    }
    Ok(())
}
