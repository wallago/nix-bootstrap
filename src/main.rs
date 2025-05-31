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
    let mut config = Config::new(config::Args::parse())?;
    let mut state = State::default();
    let mut target_dest =
        helpers::input::enter_input(None, "Enter ssh target destination (default: 127.0.0.1):")
            .await?
            .trim()
            .to_string();
    if target_dest.is_empty() {
        target_dest = "127.0.0.1".to_string();
    }
    let mut ssh_port = helpers::input::enter_input(None, "Enter ssh port (default: 22):")
        .await?
        .trim()
        .to_string();
    if ssh_port.is_empty() {
        ssh_port = "22".to_string();
    }

    // Step 2
    let mut ssh = ssh::SshSession::new(ssh_port, target_dest).await?;

    // Step 3
    let tmp_dir = helpers::temp::create_temp_dir()?;
    let nix_config_path = logic::git::initialize_nix_starter_config(&tmp_dir)?;
    config.path = Some(nix_config_path);

    // Step 4
    _ = logic::hardware::generate_target_hardware(&config, &ssh).await?;

    // Step 5
    logic::key::update_target_ssh_authorized_key(config.path.clone().unwrap())?;

    // Step 6
    state.run_nixos_anywhere = logic::deploy::run_nixos_anywhere(&config, &ssh).await?;

    // Step 7
    if state.run_nixos_anywhere {
        ssh.reconnect().await?;
    }

    // Step 8
    // logic::key::generate_age_key(&config, &ssh)?;

    // Step 9
    // 12. ❌ use `nixos-anywhere` or something else to deploy the final nix config into target
    tracing::info!("Success!");
    Ok(())
}
