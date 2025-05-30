use crate::config::Config;
use crate::helpers;
use crate::ssh::SshSession;
use anyhow::Result;

pub async fn run_nixos_anywhere(config: &Config, ssh: &SshSession) -> Result<bool> {
    if !helpers::input::ask_yes_no("Do you want to run nixos-anywhere").await? {
        tracing::warn!("Skipping nixos-anywhere");
        return Ok(false);
    }

    tracing::info!(
        "Run nixos-anywhere to {}@{} at {} for {}#{}",
        ssh.user,
        ssh.destination,
        ssh.port,
        config.path.clone().unwrap().display(),
        config.hostname
    );

    helpers::run_command(&format!(
        "nix run github:nix-community/nixos-anywhere -- --ssh-port {} --flake {}#{} {}@{}",
        ssh.port,
        config.path.clone().unwrap().display(),
        config.hostname,
        ssh.user,
        ssh.destination,
    ))?;
    Ok(true)
}
