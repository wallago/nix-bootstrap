use std::{fs::OpenOptions, io::Write, path::Path, str::FromStr};

use anyhow::{Context, Result, anyhow};
use clap::Parser;

mod helpers;
mod ssh;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Hostname (ex: nixos) of the nix config
    #[arg(short = 'd', long = "config-hostname")]
    config_hostname: String,

    /// Path (absolute) of the nix config
    #[arg(short = 'p', long = "config-path")]
    config_path: String,
}

struct Config {
    path: String,
    hostname: String,
}

impl Config {
    fn new(args: Args) -> Result<Self> {
        if !Path::new(&args.config_path).is_dir() {
            return Err(anyhow!("Config path is not a directory"));
        };
        Ok(Self {
            path: args.config_path,
            hostname: args.config_hostname,
        })
    }
}

#[derive(Default)]
struct State {
    run_nixos_anywhere: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let config = Config::new(Args::parse())?;
    let mut state = State::default();

    let mut target_dest =
        helpers::enter_input(None, "Enter ssh target destination (default: 127.0.0.1):")
            .await?
            .trim()
            .to_string();
    if target_dest.is_empty() {
        target_dest = "127.0.0.1".to_string();
    }
    let mut ssh_port = helpers::enter_input(None, "Enter ssh port (default: 22):")
        .await?
        .trim()
        .to_string();
    if ssh_port.is_empty() {
        ssh_port = "22".to_string();
    }

    let mut ssh = ssh::SshSession::new(ssh_port, target_dest).await?;

    _ = generate_target_hardware(&config, &ssh).await?;
    state.run_nixos_anywhere = run_nixos_anywhere(&config, &ssh).await?;

    if state.run_nixos_anywhere {
        ssh.reconnect().await?;
    }

    generate_age_key(&config, &ssh)?;

    tracing::info!("Success!");
    Ok(())
}

async fn generate_target_hardware(config: &Config, ssh: &ssh::SshSession) -> Result<bool> {
    if !helpers::ask_yes_no(&format!(
        "Do you want to generating hardware-configuration.nix on {}@{}",
        ssh.user, ssh.destination
    ))
    .await?
    {
        tracing::warn!("Skipping hardware-configuration generation");
        return Ok(false);
    }

    ssh.run_command("sudo nixos-generate-config --no-filesystems --root /mnt")?;
    let contents = ssh.download_file("/mnt/etc/nixos/hardware-configuration.nix")?;
    let local_path = format!(
        "{}/hosts/{}/hardware-configuration.nix",
        config.path, config.hostname
    );
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&local_path)
        .context(format!("Failed to open or apppend {}", local_path))?;
    file.write_all(&contents)?;

    Ok(true)
}

async fn run_nixos_anywhere(config: &Config, ssh: &ssh::SshSession) -> Result<bool> {
    if !helpers::ask_yes_no("Do you want to run nixos-anywhere").await? {
        tracing::warn!("Skipping nixos-anywhere");
        return Ok(false);
    }

    tracing::info!(
        "Run nixos-anywhere to {}@{} at {} for {}#{}",
        ssh.user,
        ssh.destination,
        ssh.port,
        config.path,
        config.hostname
    );

    helpers::run_command(&format!(
        "nix run github:nix-community/nixos-anywhere -- --ssh-port {} --flake {}#{} {}@{}",
        ssh.port, config.path, config.hostname, ssh.user, ssh.destination,
    ))?;
    Ok(true)
}

fn generate_age_key(config: &Config, ssh: &ssh::SshSession) -> Result<()> {
    tracing::info!(
        "Generating an age key based on the ssh key for {}@{}",
        ssh.user,
        ssh.destination
    );

    let host_age_key = age::ssh::Recipient::from_str(&ssh.pub_key)
        .map_err(|_| anyhow!("Failed to parse SSH host key into an age recipient"))?;

    tracing::debug!("ssh pub key: {}", ssh.pub_key);
    tracing::debug!("ssh pub key to age: {}", host_age_key.to_string());

    tracing::info!("Updating .sops.yaml");
    helpers::sops_update_age_key(&config.path, &config.hostname, &host_age_key.to_string())?;
    Ok(())
}
