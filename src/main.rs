use anyhow::{Result, anyhow};
use clap::{ArgAction, Parser};
use nix::{
    sys::signal::{
        Signal::{self},
        kill,
    },
    unistd::getpid,
};
use ssh2::Session;
use std::{net::TcpStream, path::Path, sync::Arc};
use tokio::{signal, sync::Mutex};

mod helpers;
mod nixos_anywhere;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Specify target_hostname of the target host to deploy the nixos config on.
    #[arg(short = 'n', long)]
    target_hostname: String,

    /// Specify ip or domain to the target host.
    #[arg(short = 'd', long)]
    target_destination: String,

    /// Specify target_user with sudo access. nix-config will be cloned to their home. [default: current user]
    #[arg(short = 'u', long, default_value_t = whoami::devicename())]
    target_user: String,

    /// Specify the full path to the ssh_key you'll use for remote access to the target during install process.
    #[arg(short = 'k', long)]
    ssh_key: String,

    /// Specify the ssh port to use for remote access. [default: 22]
    #[arg(long = "port", default_value_t = String::from("22"))]
    ssh_port: String,

    // maybe useless
    /// Specify the path to your git nixos config
    #[arg(short = 'g', long)]
    git_root: String,

    // maybe useless
    /// Specify the path to your git nixos config
    #[arg(short = 's', long)]
    nix_secrets_dir: String,

    /// Use this flag if the target machine has impermanence enabled. WARNING: Assumes /persist path.
    #[arg(long = "impermanence", action = ArgAction::SetTrue)]
    impermanence: bool,

    /// Enable debug mode.
    #[arg(long = "debug", action = ArgAction::SetTrue)]
    debug: bool,
}

struct Params {
    target_hostname: String,
    target_destination: String,
    target_user: String,
    ssh_port: String,
    ssh_key: String,
    persist_dir: String,
    nix_src_path: String,
    git_root: String,
    nix_secrets_dir: String,
    generated_hardware_config: bool,
    temp_path: String,
}

impl Params {
    fn new(args: Args, temp_path: String) -> Self {
        println!("{}", args.target_user);
        Self {
            target_hostname: args.target_hostname,
            target_destination: args.target_destination,
            target_user: args.target_user,
            ssh_port: args.ssh_port,
            ssh_key: args.ssh_key,
            persist_dir: "/persist".to_string(),
            nix_src_path: "src/nix/".to_string(),
            git_root: args.git_root,
            nix_secrets_dir: args.nix_secrets_dir,
            generated_hardware_config: true,
            temp_path,
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let temp_dir = helpers::creat_tmp_dir()?;

    let params = Params::new(
        Args::parse(),
        temp_dir.path().to_str().ok_or(anyhow!(""))?.to_string(),
    );

    let temp_dir = Arc::new(Mutex::new(Some(temp_dir)));
    let temp_dir_copy = temp_dir.clone();

    tokio::spawn(async move {
        signal::ctrl_c().await.expect("Failed to listen for Ctrl-C");
        if let Err(e) = helpers::clear_tmp_dir(temp_dir_copy).await {
            eprintln!("Failed to clear tmp dir: {e}");
        }
        std::process::exit(130);
    });

    if helpers::ask_yes_no("Run nixos-anywhere installation ?").await? {
        println!("You chose yes.");
        nixos_anywhere::setup(&params)?;
    } else {
        println!("You chose no.");
        kill(getpid(), Signal::SIGINT)?;
    }

    // if help::ask_yes_no("Generate host (ssh-based) age key ?").await? {
    //     println!("You chose yes.");
    //     // sops_genarte_host_age_key
    // } else {
    //     println!("You chose no.");
    //     kill(getpid(), Signal::SIGINT);
    // }

    // if help::ask_yes_no("Generate user age key ?").await? {
    //     println!("You chose yes.");
    //     // sops_setup_user_age_key
    // } else {
    //     println!("You chose no.");
    //     kill(getpid(), Signal::SIGINT);
    // }

    // // sops_add_creation_rules

    // if help::ask_yes_no(&format!(
    //     "Do you want to copy your full nix-config and nix-secrets to {} ?",
    //     params.target_hostname
    // ))
    // .await?
    // {
    //     println!("You chose yes.");
    //     // ssh-keyscan
    // } else {
    //     println!("You chose no.");
    //     kill(getpid(), Signal::SIGINT);
    // }

    // if help::ask_yes_no(&format!(
    //     "Do you want to commit and push the generated hardware-configuration.nix for {} to nix-config ?",
    //     params.target_hostname
    // ))
    // .await?
    // {
    //     println!("You chose yes.");
    //     // ssh-keyscan
    // } else {
    //     println!("You chose no.");
    //     kill(getpid(), Signal::SIGINT);
    // }

    helpers::clear_tmp_dir(temp_dir).await?;

    Ok(())
}

fn ssh_connection(params: Params) -> Result<()> {
    let tcp = TcpStream::connect(format!("{}:{}", params.target_destination, params.ssh_port))?;
    let mut sess = Session::new()?;
    sess.set_tcp_stream(tcp);
    sess.handshake()?;

    // Authenticate with private key
    sess.userauth_pubkey_file(&params.target_user, None, Path::new(&params.ssh_key), None)?;

    if sess.authenticated() {
        println!("SSH connection is established");
        Ok(())
    } else {
        Err(anyhow!("SSH connection failed"))
    }
}

// fn scp_upload(sess: &Session, local_path: &str, remote_path: &str) -> Result<()> {
//     // let mut local_file = File::open(local_path)?;
//     // let metadata = local_file.metadata()?;
//     // let mut remote_file = sess.scp_send(Path::new(remote_path), 0o644, metadata.len(), None)?;
//     // std::io::copy(&mut local_file, &mut remote_file)?;
//     Ok(())
// }
