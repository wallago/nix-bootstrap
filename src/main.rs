use anyhow::{Context, Result, anyhow};
use clap::Parser;
use tokio::signal;

mod helpers;
mod nixos_anywhere;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Hostname (ex: nixos) of the target host
    #[arg(short = 'n', long)]
    target_hostname: String,

    /// IP (ex: 127.0.0.1) or Domain (ex: domain.com) to the target host
    #[arg(short = 'd', long)]
    target_destination: String,

    /// User (ex: me) of the target with sudo access
    #[arg(short = 'u', long, default_value_t = whoami::devicename())]
    target_user: String,

    /// Path (absolute) to the ssh key to use for remote access.
    #[arg(short = 'k', long, default_value_t = format!("/home/{}/.ssh/id_ed25519",whoami::devicename()))]
    ssh_key_path: String,

    /// SSH port (ex: 22) of the remote access
    #[arg(long = "port", default_value_t = String::from("22"))]
    ssh_port: String,
}

struct Params {
    target_hostname: String,
    target_destination: String,
    target_user: String,
    ssh_key_path: String,
    ssh_port: String,
    persist_dir: String,
    temp_path: String,
    home_path: String,
    generated_hardware_config: bool,
}

impl Params {
    fn new(args: Args, temp_path: String) -> Result<Self> {
        Ok(Self {
            target_hostname: args.target_hostname,
            target_destination: args.target_destination,
            target_user: args.target_user,
            ssh_key_path: args.ssh_key_path,
            ssh_port: args.ssh_port,
            persist_dir: "/persist".to_string(),
            temp_path,
            home_path: dirs2::home_dir()
                .context("Error: No home directory find")?
                .to_str()
                .context("Error: Home directory parsing failed")?
                .to_string(),
            generated_hardware_config: false,
        })
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let temp_dir = helpers::creat_tmp_dir()?;

    let mut params = Params::new(
        Args::parse(),
        temp_dir
            .path()
            .to_str()
            .ok_or(anyhow!("Error: Temp directory parsing fialed"))?
            .to_string(),
    )?;

    tokio::spawn(async move {
        signal::ctrl_c().await.expect("Failed to listen for Ctrl-C");
        if let Err(e) = helpers::clear_tmp_dir(temp_dir).await {
            tracing::error!("Failed to clear tmp dir: {e}");
        }
        std::process::exit(130);
    });

    if helpers::ask_yes_no("Run nixos-anywhere installation ?").await? {
        nixos_anywhere::setup(&mut params).await?;
    } else {
        tracing::warn!("Go out of here ! Grrr");
    }

    Ok(())
}
