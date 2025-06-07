use anyhow::{Result, bail};
use clap::Parser;
use tracing::info;

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
        info!("🔸 Password has been set");
        local.git_clone_nix_stater_config()?;
        local.update_ssh_authorized_key()?;

        if hardware_config {
            local.update_hardware_config(remote.config.get_hardware_file()?)?;
        }
        if disk_device {
            local.update_disk_config(&remote.config.get_disk_device()?.name)?;
        }
        local.get_repo()?.config_changes()?;
        if !local.deploy(&remote)? {
            bail!("Couldn't continue if you don't deploy this from iso")
        }
        remote.reconnect(&local)?;
    }

    info!("🔄 Process from configured nixos");
    info!("🔸 Root privileges are available");
    // check hardware_config
    // make parser for host to see disko disk
    local.git_clone_nix_config()?;
    let age_key = remote.get_age_key()?;
    if age_key {
        local.update_sops(remote.config.get_age_key()?)?;
        local.update_encrypt_file_keys(remote.config.get_age_key()?)?;
    }
    local.get_repo()?.config_changes()?;
    // local.deploy(&remote)?;
    local.deploy_bis(&remote)?;

    // if hardware_config {
    //     info!(
    //         "Content of hardware config:\n{}",
    //         String::from_utf8(remote.config.get_hardware_file()?.to_owned())?
    //     );
    // }
    // if disk_device {
    //     info!(
    //         "Disk device selected:\n{:#?}",
    //         remote.config.get_disk_device()?
    //     );
    // }
    // if age_key {
    //     info!(
    //         "Content of sops:\n{}",
    //         String::from_utf8(remote.config.get_hardware_file()?.to_owned())?
    //     );
    // }

    Ok(info!("🚀 Enjoy !"))
}
