use anyhow::{Result, bail};
use clap::Parser;
use tracing::{info, warn};

mod helpers;
mod local;
mod params;
mod remote;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    info!("🚀 Welcome to nix-bootstrap !");
    info!("🔸 A tool to install my nix-config with sops keys update");

    let params = params::Args::parse();

    let mut local = local::Host::new()?;
    let mut remote = remote::Host::new(&params.ssh_dest, &params.ssh_port, &local)?;
    let hardware_config = remote.get_hardware_config()?;
    let disk_device = remote.get_disk_device()?;

    if params.use_iso {
        info!("🆕 Process from nix iso");
        warn!("🔸 SSH access must be available");
        warn!("🔸 Password must be set");
        local.git_clone_nix_config(true)?;
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
        remote.reconnect(&local)?;
    }

    info!("🔄 Process from configured nixos");
    warn!("🔸 SSH access must be available");
    warn!("🔸 Root privileges must be available");
    local.git_clone_nix_config(false)?;
    let age_key = remote.get_age_key()?;
    if age_key {
        local.update_sops(remote.config.get_age_key()?)?;
        local.update_encrypt_file_keys(remote.config.get_age_key()?)?;
    }
    if hardware_config {
        local.update_hardware_config(remote.config.get_hardware_file()?)?;
    }
    if disk_device {
        local.update_disk_config(&remote.config.get_disk_device()?.name)?;
    }
    local.get_repo()?.config_changes()?;
    local.deploy_nixos_rebuild(&remote)?;
    Ok(info!("🚀 Enjoy !"))
}
