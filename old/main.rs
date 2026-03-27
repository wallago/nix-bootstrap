use anyhow::{Result, bail};
use tracing::{info, warn};

mod helpers;
mod local;
mod remote;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    info!("🚀 Welcome to nix-bootstrap !");
    info!("🔸 A tool to install nixos configuration with sops keys update");
    let is_remote_system_running_on_image =
        helpers::ask_confirmation("Does remote host system is running on an installer image?")?;

    let mut local = local::Host::new()?;
    let mut remote = remote::Host::new(&local)?;
    let hardware_config = remote.get_hardware_config()?;
    let disk_device = remote.get_disk_device()?;

    if is_remote_system_running_on_image {
        info!("🆕 Remote host system is running on an image");
        warn!("🔸 SSH access must be available");
        warn!("🔸 Password must be set");
        local.set_nix_config(true, true)?;
        if hardware_config {
            local.update_hardware_config(remote.config.get_hardware_file()?)?;
        }
        if disk_device {
            local.update_disk_config(&remote.config.get_disk_device()?.name)?;
        }
        local.get_repo()?.config_changes()?;
        if !local.deploy_nixos_anywhere(&remote)? {
            bail!("Couldn't continue if you don't deploy this from iso")
        }
        helpers::ask_confirmation("Does remote host has reboot?")?;
        remote.reconnect(&local)?;
    }

    info!("🔄 Remote host system is running on an config");
    warn!("🔸 SSH access must be available");
    warn!("🔸 Root privileges must be available");

    if helpers::ask_confirmation("Do you want to use nix config locally?")? {
        local.set_nix_config(false, true)?;
    } else {
        local.set_nix_config(false, false)?;
    }

    let age_key = remote.get_age_key()?;
    if age_key {
        local.update_sops(remote.config.get_age_key()?)?;
        local.update_encrypt_file_keys()?;
    }
    if hardware_config {
        local.update_hardware_config(remote.config.get_hardware_file()?)?;
    }
    if disk_device {
        local.update_disk_config(&remote.config.get_disk_device()?.name)?;
    }
    local.get_repo()?.config_changes()?;
    local.deploy_nixos_rebuild(&remote)?;

    Ok(info!("🚀 Reboot your remote host and enjoy !"))
}
