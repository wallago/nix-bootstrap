use std::fs::OpenOptions;
use std::io::Write;

use anyhow::{Context, Result, anyhow};
use clap::Parser;
use helpers::is_ssh_key_exist_localy;
use tokio::signal;

mod helpers;
mod nixos_anywhere;
mod sops_age_key;
mod ssh;

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
    #[arg(short = 'k', long)]
    ssh_key_path: Option<String>,

    /// Password (ex: 123456) to use for remote access.
    #[arg(short = 'p', long)]
    ssh_password: Option<String>,

    /// SSH port (ex: 22) of the remote access
    #[arg(long = "port", default_value_t = String::from("22"))]
    ssh_port: String,
}

struct Params {
    target_hostname: String,
    target_destination: String,
    target_user: String,
    ssh: ssh::SSH,
    persist_dir: String,
    temp_path: String,
    home_path: String,
    generated_hardware_config: bool,
    git_dir_path: String,
}

impl Params {
    fn new(args: Args, temp_path: String) -> Result<Self> {
        Ok(Self {
            target_hostname: args.target_hostname,
            target_destination: args.target_destination.clone(),
            target_user: args.target_user.clone(),
            ssh: ssh::SSH::new(
                args.ssh_port,
                args.target_destination,
                args.ssh_password,
                args.ssh_key_path,
                args.target_user,
            )?,
            persist_dir: "/persist".to_string(),
            temp_path,
            home_path: dirs2::home_dir()
                .context("Error: No home directory find")?
                .to_str()
                .context("Error: Home directory parsing failed")?
                .to_string(),
            generated_hardware_config: false,
            git_dir_path: git2::Repository::discover(".")?
                .path()
                .to_str()
                .ok_or(anyhow!("Error: Git path parsing to string failed"))?
                .to_string(),
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

    sops_age_key::setup(&mut params).await?;

    if helpers::ask_yes_no(&format!(
        "Do you want to copy your full nix-config to {} ?",
        params.target_hostname
    ))
    .await?
    {
        tracing::info!(
            "Adding ssh host fingerprint at {} to ~/.ssh/known_hosts",
            params.target_hostname
        );

        let home_ssh_path = format!("{}/.ssh/known_hosts", params.home_path);
        tracing::info!(
            "Adding ssh host fingerprint at {} to {}",
            params.target_destination,
            home_ssh_path
        );

        let host_entry = format!(
            "[{}]:{} {}",
            params.target_destination, params.ssh.port, params.ssh.host_key
        );
        if !is_ssh_key_exist_localy(&params, &host_entry)? {
            let mut file = OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(&home_ssh_path)
                .context(format!(
                    "Error: Failed to open or apppend {}",
                    home_ssh_path
                ))?;
            writeln!(file, "{}", host_entry).context(format!(
                "Error: Failed to add line {} into {}",
                params.ssh.host_key, home_ssh_path
            ))?;
        }
        params.ssh.upload(
            &params.ssh.get_sftp()?,
            &format!("{}/../nix-config", params.git_dir_path),
            &format!("/home/{}", params.target_hostname),
        )?;

        if helpers::ask_yes_no("Do you want to rebuild immediately ?").await? {
            tracing::info!("Rebuilding nix-config on {}", params.target_hostname);
            params.ssh.run_command(&format!(
                "cd /home/{}/nix-config && sudo nixos-rebuild --flake .#{} switch",
                params.target_hostname, params.target_hostname
            ))?;
        } else {
            tracing::warn!("Go out of here ! Grrr");
        }
    } else {
        tracing::info!("NixOS was successfully installed !");
        tracing::info!(
            "Post-install config build instructions:\nTo copy nix-config from this machine to the {}, run the following command.\nscp - \nTo rebuild, sign into {} and run the following command.\ncd nix-config\nsudo nixos-rebuild --show-trace --flake .#{} switch",
            params.target_hostname,
            params.target_hostname,
            params.target_hostname
        );
    }

    tracing::info!("Success!");

    Ok(())
}
