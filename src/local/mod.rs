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
        let ssh = ssh::Info::new(home_dir.join(".ssh/known_hosts"));
        Ok(Self { repo: None, ssh })
    }

    pub fn set_nix_config(&mut self, use_iso: bool, use_path: bool) -> Result<()> {
        self.repo = Some(Repo::clone_nix_config(use_iso, use_path)?);
        Ok(())
    }

    pub fn get_repo(&self) -> Result<&Repo> {
        Ok(self
            .repo
            .as_ref()
            .ok_or_else(|| anyhow!("Git repo not seems to be cloned"))?)
    }
}
