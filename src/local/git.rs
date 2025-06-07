use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, anyhow};
use dialoguer::{Confirm, Select, theme::ColorfulTheme};
use git2::Repository;
use tempfile::TempDir;
use tracing::info;

use crate::helpers;

pub struct Repo {
    pub git: Repository,
    pub path: PathBuf,
    #[allow(dead_code)]
    tmp_dir: TempDir, // Keep tempdir alive
    pub host: String,
}

impl Repo {
    pub fn clone_nix_stater_config() -> Result<Self> {
        info!("📂 Clone nix-stater-config git repository");
        let (repo, tmp_dir) = helpers::git::clone_repository("nix-starter-config")?;
        let repo_dir = repo.path().to_path_buf();
        let repo_path = repo_dir
            .parent()
            .context("Could not get parent path of cloned git repository")?;
        Ok(Self {
            git: repo,
            path: repo_path.to_path_buf(),
            tmp_dir,
            host: String::from("plankton"),
        })
    }

    pub fn clone_nix_config() -> Result<Self> {
        info!("📂 Clone nix-config git repository ");
        let (repo, tmp_dir) = helpers::git::clone_repository("nix-config")?;
        let repo_dir = repo.path().to_path_buf();
        let repo_path = repo_dir
            .parent()
            .context("Could not get parent path of cloned git repository")?;
        Ok(Self {
            git: repo,
            path: repo_path.to_path_buf(),
            tmp_dir,
            host: Self::get_config_host(repo_path)?,
        })
    }

    pub fn get_config_host(repo_path: &Path) -> Result<String> {
        let hosts =
            serde_json::from_str::<Vec<String>>(&helpers::command::run_with_stdout(&format!(
                " nix eval --json {}#nixosConfigurations --apply builtins.attrNames",
                repo_path.display()
            ))?)?;
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select a target block device?")
            .items(&hosts)
            .interact()?;
        let host = hosts
            .get(selection)
            .ok_or_else(|| anyhow!("Selected host doesn't be found"))?;
        Ok(host.to_string())
    }

    pub fn config_changes(&self) -> Result<()> {
        info!("📝 Untrack config changes");
        let files = helpers::git::untrack_changes(&self.git)?;
        files.iter().for_each(|file| println!("🔸 {file}"));
        if Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Do you want to see the detail of those changes?")
            .interact()?
        {
            for file in files {
                info!(
                    "🔸 {}:\n{}",
                    file,
                    fs::read_to_string(format!("{}/{}", self.path.display(), file))?
                );
            }
        }
        Ok(())
    }
}
