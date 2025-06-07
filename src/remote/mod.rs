use anyhow::{Result, anyhow};
use dialoguer::{Confirm, Select, theme::ColorfulTheme};
use ssh2::Session;
use tracing::{info, warn};

use crate::{helpers::disk::DiskDevices, local};

mod config;
mod ssh;

pub struct Host {
    pub destination: String,
    pub user: String,
    pub port: String,
    ssh: Session,
    pub ssh_pk: String,
    pub config: config::Config,
}

impl Host {
    pub fn new(destination: &str, port: &u32, local: &local::Host) -> Result<Self> {
        let port = port.to_string();
        let (ssh, ssh_pk, user) = Self::connect(&destination, &port, local)?;
        Ok(Self {
            user,
            destination: destination.to_owned(),
            port,
            ssh,
            ssh_pk,
            config: config::Config::default(),
        })
    }

    pub fn get_hardware_config(&mut self) -> Result<bool> {
        if !Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(format!("Do you want to get hardware configuration?",))
            .interact()?
        {
            warn!("❗ Skipping hardware-configuration part");
            return Ok(false);
        }

        info!("🔧 Get hardware configuration");
        self.run_command("nixos-generate-config --no-filesystems --root /tmp")?;
        self.config.hardware_file =
            Some(self.download_file("/tmp/etc/nixos/hardware-configuration.nix")?);
        Ok(true)
    }

    pub fn get_disk_device(&mut self) -> Result<bool> {
        if !Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Do you want to select a disk device?")
            .interact()?
        {
            warn!("❗ Skipping disk device selection");
            return Ok(false);
        }

        let disk_devices = serde_json::from_str::<DiskDevices>(
            &self.run_command("lsblk -d -J -o NAME,SIZE,MODEL,MOUNTPOINT")?,
        )?;
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select a target block device?")
            .items(
                &disk_devices
                    .blockdevices
                    .iter()
                    .map(|disk_device| disk_device.get_info())
                    .collect::<Vec<String>>(),
            )
            .interact()?;
        let disk_device = disk_devices
            .blockdevices
            .get(selection)
            .ok_or_else(|| anyhow!("Couldn't found selected disk found"))?;
        self.config.disk_device = Some(disk_device.to_owned());
        Ok(true)
    }

    pub fn get_age_key(&mut self) -> Result<bool> {
        if !Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Do you want to get age key?")
            .interact()?
        {
            warn!("❗ Skipping age key part");
            return Ok(false);
        }

        info!("🔑 Get age key");
        self.config.age_pk = Some(ssh_to_age::convert::ssh_public_key_to_age(&self.ssh_pk)?);
        Ok(true)
    }
}
