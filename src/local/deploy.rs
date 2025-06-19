use anyhow::Result;
use tracing::{info, warn};

use crate::{helpers, remote};

impl super::Host {
    pub fn deploy_nixos_anywhere(&self, remote: &remote::Host) -> Result<bool> {
        if !helpers::ask_confirmation("Do you want to run nixos-anywhere?")? {
            warn!("❗ Skipping deployments via nixos-anywhere");
            return Ok(false);
        }

        info!("🚀 Deploying via nixos-anywhere");
        let repo = self.get_repo()?;
        let command = format!(
            "nix run github:nix-community/nixos-anywhere -- --ssh-port {} --flake {}#{} --target-host {}@{}",
            remote.port,
            repo.path.display(),
            repo.host,
            remote.user,
            remote.destination,
        );
        tracing::info!("🔸 {command}");

        loop {
            match helpers::command::run(&command) {
                Ok(_) => return Ok(true),
                Err(err) => {
                    if !helpers::ask_confirmation("Do you want to retry?")? {
                        return Err(err);
                    }
                }
            }
        }
    }

    pub fn deploy_nixos_rebuild(&self, remote: &remote::Host) -> Result<bool> {
        if !helpers::ask_confirmation("Do you want to run nixos-rebuild?")? {
            warn!("❗ Skipping deployments via nixos-rebuild");
            return Ok(false);
        }

        info!("🚀 Deploying nix-config via nixos-rebuild");
        let repo = self.get_repo()?;
        let command = format!(
            "NIX_SSHOPTS=\"-p {}\" nixos-rebuild switch --flake {}#{} --build-host {}@{} --target-host {}@{} --use-substitutes --sudo --ask-sudo-password",
            remote.port,
            repo.path.display(),
            repo.host,
            remote.user,
            remote.destination,
            remote.user,
            remote.destination,
        );
        tracing::info!("🔸 {command}");
        loop {
            match helpers::command::run(&command) {
                Ok(_) => return Ok(true),
                Err(err) => {
                    if !helpers::ask_confirmation("Do you want to retry?")? {
                        return Err(err);
                    }
                }
            }
        }
    }
}
