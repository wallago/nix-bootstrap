use anyhow::Result;
use clap::Parser;
use config::Config;

mod config;
mod helpers;
mod logic;
mod ssh;

#[derive(Default)]
struct State {
    run_nixos_anywhere: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    // Step 1
    let args = config::Args::parse();
    let mut config = Config::new()?;
    let mut state = State::default();
    // Step 2
    let mut ssh = ssh::SshSession::new(&args)?;

    // Step 3
    let tmp_dir = helpers::temp::create_temp_dir()?;
    let nix_config_path = logic::git::initialize_nix_starter_config(&tmp_dir)?;
    config.path = Some(nix_config_path);

    // Step 4
    _ = logic::hardware::generate_target_hardware(&config, &ssh)?;

    // Step 5
    logic::key::update_target_ssh_authorized_key(config.path.clone().unwrap())?;
    let target_block_device = logic::select_target_block_device(&ssh)?;
    config.block_device = Some(target_block_device);

    // Step 6
    state.run_nixos_anywhere = logic::deploy::run_nixos_anywhere(&config, &ssh).await?;

    // Step 7
    if state.run_nixos_anywhere {
        ssh.reconnect();
    }

    // Step 8
    // logic::key::generate_age_key(&config, &ssh)?;

    // Step 9
    // 12. ❌ use `nixos-anywhere` or something else to deploy the final nix config into target
    tracing::info!("Success!");
    Ok(())
}
