use anyhow::Result;
use clap::Parser;
use tracing::info;

mod helpers;
mod local;
mod params;
mod remote;
mod state;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    info!("🚀 Welcome to nix-bootstrap !");
    info!("A tool to install my nix-config with sops keys update");

    let params = params::Args::parse();
    let mut state = state::State::default();

    let mut local = local::Host::new()?;
    let mut remote = remote::Host::new(params.ssh_dest, params.ssh_port, &local)?;

    if !params.use_sudo {
        local.git_clone_nix_stater_config()?;
    }

    state.get_hardware_config = remote.get_hardware_config()?;
    if state.get_hardware_config {
        if local.update_hardware_config(remote.config.hardware_file.as_ref())? {
            remote.get_disk_device()?;
            local.update_disk_config(remote.config.disk_device.as_ref())?;
        }
    }

    Ok(info!("🚀 Enjoy !"))
}
