use anyhow::Result;
use dialoguer::{Confirm, theme::ColorfulTheme};
use tracing::{info, warn};

use crate::{helpers, remote};

impl super::Host {
    pub fn deploy(&self, remote: &remote::Host) -> Result<bool> {
        if !Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Do you want to run nixos-anywhere?")
            .interact()?
        {
            warn!("❗ Skipping deployments via nixos-anywhere");
            return Ok(false);
        }

        info!("🚀 Deploying via nixos-anywhere");
        let repo = self.get_repo()?;
        helpers::command::run(&format!(
            "nix run github:nix-community/nixos-anywhere -- --ssh-port {} --flake {}#{} --target-host {}@{}",
            remote.port,
            repo.path.display(),
            repo.host,
            remote.user,
            remote.destination,
        ))?;

        Ok(true)
    }

    pub fn deploy_bis(&self, remote: &remote::Host) -> Result<bool> {
        if !Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Do you want to run nixos-rebuild?")
            .interact()?
        {
            warn!("❗ Skipping deployments via nixos-rebuild");
            return Ok(false);
        }

        info!("🚀 Deploying nix-config via nixos-rebuild");
        let repo = self.get_repo()?;
        // helpers::command::run(&format!(
        //     "NIX_SSHOPTS=\"-p {}\" nixos-rebuild switch --flake {}#{} --build-host {}@{} --target-host {}@{} --use-remote-sudo",
        //     remote.port,
        //     repo.path.display(),
        //     repo.host,
        //     remote.user,
        //     remote.destination,
        //     remote.user,
        //     remote.destination,
        // ))?;
        helpers::command::run(&format!(
            "NIX_SSHOPTS=\"-p {}\" nixos-rebuild switch --flake {}#{} --target-host {}@{}",
            remote.port,
            repo.path.display(),
            repo.host,
            remote.user,
            remote.destination,
        ))?;

        Ok(true)
    }
}
