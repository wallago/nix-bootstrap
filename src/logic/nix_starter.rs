use std::path::PathBuf;

use anyhow::{Context, Result};
use tempfile::tempdir;

pub fn initialize_nix_starter_config() -> Result<PathBuf> {
    let temp_dir = tempdir().context("Failed to create temp directory")?;
    tracing::info!(
        "Temporary directory created at: {}",
        temp_dir.path().display()
    );

    let repo = git2::Repository::clone(
        "https://github.com/wallago/nix-starter-configs.git",
        temp_dir.path(),
    )?;

    Ok(repo.path().to_path_buf())
}
