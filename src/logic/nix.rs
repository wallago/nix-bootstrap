use std::path::PathBuf;

use anyhow::Result;
use dialoguer::{Confirm, Select, theme::ColorfulTheme};

use crate::{config, helpers, ssh::SshSession};

pub fn select_config_host(config_path: &PathBuf) -> Result<String> {
    let hosts = serde_json::from_str::<Vec<String>>(&helpers::run_command(&format!(
        " nix eval --json {}#nixosConfigurations --apply builtins.attrNames",
        config_path.display()
    ))?)?;
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select a target block device?")
        .items(&hosts)
        .interact()?;

    let selected_host = hosts
        .get(selection)
        .ok_or_else(|| anyhow::anyhow!("Selected host doesn't be found"))?
        .clone();

    tracing::info!("Selected host: {}", selected_host);
    Ok(selected_host.to_string())
}

pub fn run_nixos_rebuild(config: &config::Config, ssh: &SshSession) -> Result<bool> {
    if !Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(format!(
            "Do you want to run nixos-rebuild on {}@{}",
            ssh.user, ssh.destination
        ))
        .interact()?
    {
        tracing::warn!("Skipping nixos-rebuild");
        return Ok(false);
    }

    let config_path = config.path.clone().unwrap();
    let host = config.host.clone().unwrap();
    let ssh_host = format!("{}@{}", ssh.user, ssh.destination);
    tracing::info!(
        "Run nixos-rebuild to {ssh_host} at {} for {}#{host}",
        ssh.port,
        config.path.clone().unwrap().display(),
    );
    helpers::run_command_with_stdout(&format!(
        "NIX_SSHOPTS=\"-p {}\" nixos-rebuild  switch --flake {}#{host} --build-host {ssh_host} --target-host {ssh_host} --use-remote-sudo",
        ssh.port,
        config_path.display(),
    ))?;
    Ok(true)
}
