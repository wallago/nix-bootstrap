use std::fs::OpenOptions;
use std::io::Write;

use crate::{config::Config, ssh::SshSession};
use anyhow::{Context, Result};
use dialoguer::Confirm;
use dialoguer::theme::ColorfulTheme;

pub fn generate_target_hardware_config(ssh: &SshSession) -> Result<Option<Vec<u8>>> {
    tracing::info!("Generate hardware configuration on the target");
    if !Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(format!(
            "Do you want to generating hardware-configuration.nix on {}@{}",
            ssh.user, ssh.destination
        ))
        .interact()?
    {
        tracing::warn!("Skipping hardware-configuration generation");
        return Ok(None);
    }

    ssh.run_command("nixos-generate-config --no-filesystems --root /tmp")?;
    let contents = ssh.download_file("/tmp/etc/nixos/hardware-configuration.nix")?;
    Ok(Some(contents))
}

pub fn write_target_hardware_starter_config(config: &Config) -> Result<()> {
    tracing::info!("Write hardware configuration inside starter configuration");
    let local_path = format!(
        "{}/nixos/hardware-configuration.nix",
        config.path.clone().unwrap().display()
    );
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&local_path)
        .context(format!("Failed to open or apppend {}", local_path))?;
    file.write_all(&config.hardware_config.clone().unwrap())?;
    Ok(())
}

pub fn write_target_hardware_config(config: &Config, host: &str) -> Result<()> {
    tracing::info!("Write hardware configuration inside configuration");
    let local_path = format!(
        "{}/hosts/{host}/hardware-configuration.nix",
        config.path.clone().unwrap().display()
    );
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&local_path)
        .context(format!("Failed to open or apppend {}", local_path))?;
    file.write_all(&config.hardware_config.clone().unwrap())?;
    Ok(())
}
