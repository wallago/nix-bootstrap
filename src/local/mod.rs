use std::fs::{self};

use anyhow::{Result, anyhow};

use crate::local::{git::Repo, ssh::Info};

mod deploy;
mod git;
mod ssh;
mod update;

pub struct Host {
    repo: Option<Repo>,
    pub ssh: Info,
}

impl Host {
    pub fn new() -> Result<Self> {
        let home_dir =
            dirs2::home_dir().ok_or_else(|| anyhow!("Could not find local home directory"))?;
        let pk = fs::read_to_string(home_dir.join(".ssh/id_ed25519.pub"))?;

        let ssh = ssh::Info::new(
            home_dir.join(".ssh/id_ed25519.pub"),
            home_dir.join(".ssh/id_ed25519"),
            home_dir.join(".ssh/known_hosts"),
            pk,
        );
        Ok(Self { repo: None, ssh })
    }

    pub fn git_clone_nix_stater_config(&mut self) -> Result<()> {
        self.repo = Some(Repo::clone_nix_stater_config()?);
        Ok(())
    }

    pub fn git_clone_nix_config(&mut self) -> Result<()> {
        self.repo = Some(Repo::clone_nix_config()?);
        Ok(())
    }

    pub fn get_repo(&self) -> Result<&Repo> {
        Ok(self
            .repo
            .as_ref()
            .ok_or_else(|| anyhow!("Git repo not seems to be cloned"))?)
    }
}
