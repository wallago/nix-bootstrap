use std::path::PathBuf;

use anyhow::{Context, Result, anyhow, bail};
use git2::Repository;
use tracing::info;

use crate::helpers;

struct Repo {
    is_starter: bool,
    git: Repository,
    path: Path,
}

pub struct Host {
    repo: Option<Repo>,
    pub ssh_pk_path: Path,
    pub ssh_sk_path: Path,
}

impl Host {
    pub fn new() -> Result<Self> {
        let home_dir =
            dirs2::home_dir().ok_or_else(|| anyhow!("Could not find local home directory"))?;

        Ok(Self {
            repo: None,
            ssh_pk_path: home_dir.join(".ssh/id_ed25519.pub"),
            ssh_sk_path: home_dir.join(".ssh/id_ed25519"),
        })
    }

    pub fn git_clone_nix_stater_config(&mut self) -> Result<()> {
        info!("📂 Clone nix-stater-config git repository");
        let repo = helpers::git_clone_repository("nix-stater-config")?;
        let repo_path = repo
            .path()
            .parent()
            .context("Could not get parent path of cloned git repository")?;
        self.repo = Some(Repo {
            is_starter: true,
            git: repo,
            path: repo_path,
        });
        Ok(info!(
            "📂 Git repository is available at {}",
            repo_path.display()
        ))
    }

    pub fn git_clone_nix_config(&mut self) -> Result<()> {
        info!("📂 Clone nix-config git repository ");
        let repo = helpers::git_clone_repository("nix-config")?;
        let repo_path = repo
            .path()
            .parent()
            .context("Could not get parent path of cloned git repository")?;
        self.repo = Some(Repo {
            is_starter: false,
            git: repo,
            path: repo_path,
        });
        Ok(info!(
            "📂 Git repository is available at {}",
            repo_path.display()
        ))
    }

    pub fn update_hardware_config(&self, content: &[u8], host: Option<&str>) -> Result<()> {
        let repo = self
            .repo
            .as_ref()
            .ok_or_else(|| anyhow!("Git repo not seems to be cloned"))?;

        let hardware_config_path = match repo.is_starter {
            true => format!("{}/nixos/hardware-configuration.nix", repo.path),
            false => {
                let host = host.ok_or_else(|| anyhow!("Host isn't set"))?;
                format!("{}/hosts/{host}/hardware-configuration.nix", repo.path)
            }
        };
        // let local_path = ;
        // let mut file = OpenOptions::new()
        //     .create(true)
        //     .write(true)
        //     .truncate(true)
        //     .open(&local_path)
        //     .context(format!("Failed to open or apppend {}", local_path))?;
        // file.write_all(&config.hardware_config.clone().unwrap())?;

        Ok(())
    }
}
