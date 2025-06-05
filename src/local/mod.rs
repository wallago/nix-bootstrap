use std::{fs, path::PathBuf};

use anyhow::{Context, Result, anyhow};
use git2::Repository;
use tracing::{info, warn};

use crate::helpers::{self, DiskDevice};

struct Repo {
    is_starter: bool,
    git: Repository,
    path: PathBuf,
}

pub struct Host {
    repo: Option<Repo>,
    pub ssh_pk_path: PathBuf,
    pub ssh_sk_path: PathBuf,
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
        let repo_dir = repo.path().to_path_buf();
        let repo_path = repo_dir
            .parent()
            .context("Could not get parent path of cloned git repository")?;
        self.repo = Some(Repo {
            is_starter: true,
            git: repo,
            path: repo_path.to_path_buf(),
        });
        Ok(info!(
            "📂 Git repository is available at {}",
            repo_path.display()
        ))
    }

    pub fn git_clone_nix_config(&mut self) -> Result<()> {
        info!("📂 Clone nix-config git repository ");
        let repo = helpers::git_clone_repository("nix-config")?;
        let repo_dir = repo.path().to_path_buf();
        let repo_path = repo_dir
            .parent()
            .context("Could not get parent path of cloned git repository")?;
        self.repo = Some(Repo {
            is_starter: false,
            git: repo,
            path: repo_path.to_path_buf(),
        });
        Ok(info!(
            "📂 Git repository is available at {}",
            repo_path.display()
        ))
    }

    pub fn update_hardware_config(&self, contents: Option<&Vec<u8>>) -> Result<bool> {
        info!("✏️ Update hardware config");
        let repo = self
            .repo
            .as_ref()
            .ok_or_else(|| anyhow!("Git repo not seems to be cloned"))?;
        let hardware_config_path = match repo.is_starter {
            true => format!("{}/nixos/hardware-configuration.nix", repo.path.display()),
            false => {
                warn!("⚠️Not available for nix-config (only for the stater)");
                return Ok(false);
            }
        };
        fs::write(
            hardware_config_path,
            contents.ok_or_else(|| anyhow!("Contents isn't given"))?,
        )?;
        Ok(true)
    }

    pub fn update_disk_config(&self, contents: Option<&DiskDevice>) -> Result<bool> {
        info!("✏️ Update disk config");
        let repo = self
            .repo
            .as_ref()
            .ok_or_else(|| anyhow!("Git repo not seems to be cloned"))?;
        let disk_config_path = match repo.is_starter {
            true => format!("{}/nixos/disk-device.txt", repo.path.display()),
            false => {
                warn!("⚠️Not available for nix-config (only for the stater)");
                return Ok(false);
            }
        };
        fs::write(
            disk_config_path,
            format!(
                "/dev/{}",
                contents
                    .ok_or_else(|| anyhow!("Contents isn't given"))?
                    .name
            ),
        )?;
        Ok(true)
    }
}
