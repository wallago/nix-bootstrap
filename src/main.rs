use anyhow::{Result, anyhow};
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
    let remote = remote::Host::new(params.ssh_dest, params.ssh_port, &local)?;

    if !params.use_sudo {
        local.git_clone_nix_stater_config()?;
    }

    state.get_hardware_config = remote.get_hardware_config()?;
    if state.get_hardware_config {
        local.update_hardware_config(
            &remote
                .config
                .hardware_file
                .ok_or_else(|| anyhow!("Hardware file isn't defined"))?,
            None,
        )?;

        //     let target_block_device =
        //         logic::disk::select_target_block_device(&ssh, &config.path.clone().unwrap())?;
        //     config.block_device = Some(target_block_device);
    }

    Ok(info!("🚀 Enjoy !"))
}
