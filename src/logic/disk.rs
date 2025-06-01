use std::{fs, path::PathBuf};

use anyhow::Result;
use dialoguer::{Select, theme::ColorfulTheme};
use serde::{Deserialize, Serialize};

use crate::ssh::SshSession;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct BlockDevices {
    blockdevices: Vec<BlockDevice>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockDevice {
    name: String,
    size: String,
    model: Option<String>,
    mountpoint: Option<String>,
}

impl BlockDevice {
    fn get_info(&self) -> String {
        format!(
            "{} (size: {} / model: {:?} / mountpoint: {:?})",
            self.name, self.size, self.model, self.mountpoint
        )
    }
}

pub fn select_target_block_device(ssh: &SshSession, config_path: &PathBuf) -> Result<BlockDevice> {
    let target_block_devices = serde_json::from_str::<BlockDevices>(
        &ssh.run_command("lsblk -d -J -o NAME,SIZE,MODEL,MOUNTPOINT")?,
    )?;
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select a target block device?")
        .items(
            &target_block_devices
                .blockdevices
                .iter()
                .map(|block_device| block_device.get_info())
                .collect::<Vec<String>>(),
        )
        .interact()?;

    let selected_target_block_device: BlockDevice = target_block_devices
        .blockdevices
        .get(selection)
        .ok_or_else(|| anyhow::anyhow!("Selected block device doesn't be found"))?
        .clone();

    tracing::info!(
        "Selected target block device: {}",
        selected_target_block_device.name
    );

    write_block_device(config_path, &selected_target_block_device.name)?;

    Ok(selected_target_block_device)
}

fn write_block_device(config_path: &PathBuf, block_device: &str) -> Result<()> {
    let ssh_pk_path = format!("{}/nixos/disk-device.txt", config_path.display(),);
    Ok(fs::write(ssh_pk_path, format!("/dev/{}", block_device))?)
}
