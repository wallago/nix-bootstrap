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
    let nix_config_path = logic::nix_starter::initialize_nix_starter_config()?;
    config.path = Some(nix_config_path);

    // Step 4
    _ = logic::hardware::generate_target_hardware(&config, &ssh).await?;

    // 7. ❌ update nix-starter-config to know host ssh pub key
    state.run_nixos_anywhere = logic::deploy::run_nixos_anywhere(&config, &ssh).await?;

    // 9. ✅ reconnect ssh connection with the new config
    if state.run_nixos_anywhere {
        ssh.reconnect().await?;
    }

    // 10. ✅ get ssh ed25519 key and generate `age` key
    // 11. ✅ update `.sops.yaml` and `ssh_host_ed25519_key.pub`
    logic::key::generate_age_key(&config, &ssh)?;

    // 12. ❌ use `nixos-anywhere` or something else to deploy the final nix config into target
    tracing::info!("Success!");
    Ok(())
}
