use anyhow::Result;
use clap::Parser;
use config::Config;

mod config;
mod helpers;
mod logic;
mod ssh;

#[derive(Default)]
struct State {
    generate_hardware_config: bool,
    run_nixos_anywhere: bool,
}

fn main() -> Result<()> {
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
    let target_hardware_config = logic::hardware::generate_target_hardware_config(&ssh)?;
    state.generate_hardware_config = target_hardware_config.is_some();
    config.hardware_config = target_hardware_config;
    if state.generate_hardware_config {
        logic::hardware::write_target_hardware_starter_config(&config)?;
        let target_block_device =
            logic::disk::select_target_block_device(&ssh, &config.path.clone().unwrap())?;
        config.block_device = Some(target_block_device);
    }

    // Step 5
    logic::key::update_target_ssh_authorized_key(config.path.clone().unwrap())?;

    // Step 6
    state.run_nixos_anywhere = logic::deploy::run_nixos_anywhere(&config, &ssh)?;
    if state.run_nixos_anywhere {
        ssh.reconnect()?;
    }

    // Step 7
    let tmp_dir = helpers::temp::create_temp_dir()?;
    let nix_config_path = logic::git::initialize_nix_config(&tmp_dir)?;
    config.path = Some(nix_config_path);

    // Step 8
    let host = logic::nix::select_config_host(&config.path.clone().unwrap())?;
    config.host = Some(host);

    // Step 9
    if state.generate_hardware_config {
        logic::hardware::write_target_hardware_config(&config, &config.host.clone().unwrap())?;
    }

    // Step 10
    let target_pk_age = logic::key::generate_age_key(&config, &ssh)?;
    config.pk_age = Some(target_pk_age);

    // Step 11
    logic::key::update_encrypted_file_keys(&config)?;

    // Step 12
    logic::nix::run_nixos_rebuild(&config, &ssh)?;

    // Step 13
    logic::git::untracked_changes(&config)?;
    // maybe suggest a git push :) cause the repo will be erase at the end
    tracing::info!("Success!");
    Ok(())
}
