use anyhow::Result;
use clap::Parser;
use tracing::info;

mod helpers;
mod local;
mod params;
mod remote;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    info!("🚀 Welcome to nix-bootstrap !");
    info!("A tool to install my nix-config with sops keys update");

    let params = params::Args::parse();

    let mut local = local::Host::new()?;
    let mut remote = remote::Host::new(&params.ssh_dest, &params.ssh_port, &local)?;
    let hardware_config = remote.get_hardware_config()?;
    let disk_device = remote.get_disk_device()?;

    if params.use_iso {
        info!("🆕 Process from nix iso");
        info!("🔸 Password has been set");
        local.git_clone_nix_stater_config()?;
        local.update_ssh_public_key()?;

        if hardware_config {
            local.update_hardware_config(remote.config.get_hardware_file()?)?;
        }
        if disk_device {
            local.update_disk_config(&remote.config.get_disk_device()?.name)?;
        }
        local.deploy_nix_stater_config(&remote)?;
        remote.reconnect(&local)?;
    }

    info!("🔄 Process from configured nixos");
    info!("🔸 Root privileges are available");
    // check hardware_config
    // make parser for host to see disko disk
    // sops update secrets to accecpt it
    local.git_clone_nix_config()?;
    local.get_config_host()?;
    let age_key = remote.get_age_key()?;
    if age_key {
        local.update_sops(remote.config.get_age_key()?)?;
    }
    local.deploy_nix_config(&remote)?;

    if hardware_config {
        info!(
            "Content of hardware config:\n{}",
            String::from_utf8(remote.config.get_hardware_file()?.to_owned())?
        );
    }
    if disk_device {
        info!(
            "Disk device selected:\n{:#?}",
            remote.config.get_disk_device()?
        );
    }
    if age_key {
        info!(
            "Content of sops:\n{}",
            String::from_utf8(remote.config.get_hardware_file()?.to_owned())?
        );
    }

    Ok(info!("🚀 Enjoy !"))
}
