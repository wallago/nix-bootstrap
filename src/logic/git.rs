use std::{path::PathBuf, str::FromStr};

use anyhow::Result;
use tempfile::TempDir;

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
