use std::fs::OpenOptions;
use std::io::Write;

use crate::helpers;
use crate::{config::Config, ssh::SshSession};
use anyhow::{Context, Result};
use dialoguer::Confirm;
use dialoguer::theme::ColorfulTheme;

pub fn generate_target_hardware(config: &Config, ssh: &SshSession) -> Result<bool> {
    if Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(format!(
            "Do you want to generating hardware-configuration.nix on {}@{}",
            ssh.user, ssh.destination
        ))
        .interact()?
    {
        tracing::warn!("Skipping hardware-configuration generation");
        return Ok(false);
    }

    ssh.run_command("sudo nixos-generate-config --no-filesystems --root /mnt")?;
    let contents = ssh.download_file("/mnt/etc/nixos/hardware-configuration.nix")?;
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
    file.write_all(&contents)?;

    Ok(true)
}
