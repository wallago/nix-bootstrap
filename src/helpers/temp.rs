use anyhow::{Context, Result};
use tempfile::{TempDir, tempdir};

pub fn create_temp_dir() -> Result<TempDir> {
    let tmp_dir = tempdir().context("Failed to create temp directory")?;
    tracing::info!(
        "Temporary directory created at: {}",
        tmp_dir.path().display()
    );

    Ok(tmp_dir)
}
