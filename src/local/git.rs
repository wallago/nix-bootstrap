use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, anyhow};
use dialoguer::{Select, theme::ColorfulTheme};
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
    pub fn clone_nix_config(use_iso: bool) -> Result<Self> {
        info!("📂 Clone nix-config git repository ");
        let (repo, tmp_dir) = helpers::git::clone_repository("nix-config")?;
        let repo_dir = repo.path().to_path_buf();
        let repo_path = repo_dir
            .parent()
            .context("Could not get parent path of cloned git repository")?;
        let host = Self::get_config_host(repo_path, use_iso)?;
        Ok(Self {
            git: repo,
            path: repo_path.to_path_buf(),
            tmp_dir,
            host,
        })
    }

    fn get_config_host(repo_path: &Path, use_iso: bool) -> Result<String> {
        let mut hosts =
            serde_json::from_str::<Vec<String>>(&helpers::command::run_with_stdout(&format!(
                " nix eval --json {}#nixosConfigurations --apply builtins.attrNames",
                repo_path.display()
            ))?)?;
        if use_iso {
            hosts = hosts
                .into_iter()
                .filter(|host| host.starts_with("plankton"))
                .collect::<Vec<String>>();
        }
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select a config host?")
            .items(&hosts)
            .interact()?;
        let host = hosts
            .get(selection)
            .ok_or_else(|| anyhow!("Selected host doesn't be found"))?;
        Ok(host.to_string())
    }

    pub fn get_host(&self) -> &str {
        if self.host.starts_with("plankton") {
            "plankton"
        } else {
            &self.host
        }
    }

    pub fn config_changes(&self) -> Result<()> {
        info!("📝 Untrack config changes");
        let files = helpers::git::untrack_changes(&self.git)?;
        files.iter().for_each(|file| println!("🔸 {file}"));
        if helpers::ask_confirmation("Do you want to see the detail of those changes?")? {
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
