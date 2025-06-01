use crate::config::Config;
use crate::helpers;
use crate::ssh::SshSession;
use anyhow::Result;
use dialoguer::Confirm;
use dialoguer::theme::ColorfulTheme;

pub fn run_nixos_anywhere(config: &Config, ssh: &SshSession) -> Result<bool> {
    if !Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Do you want to run nixos-anywhere?")
        .interact()?
    {
        tracing::warn!("Skipping nixos-anywhere");
        return Ok(false);
    }

    tracing::info!(
        "Run nixos-anywhere to {}@{} at {} for {}#{}",
        ssh.user,
        ssh.destination,
        ssh.port,
        config.path.clone().unwrap().display(),
        "plankton"
    );

    helpers::run_command_with_stdout(&format!(
        "nix run github:nix-community/nixos-anywhere -- --ssh-port {} --flake {}#{} {}@{}",
        ssh.port,
        config.path.clone().unwrap().display(),
        "plankton",
        ssh.user,
        ssh.destination,
    ))?;
    Ok(true)
}
