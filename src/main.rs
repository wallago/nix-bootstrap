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

    /// User (ex: me) with sudo access
    #[arg(short = 'u', long, default_value_t = whoami::devicename())]
    target_user: String,
}

struct Params {
    target_hostname: String,
    target_destination: String,
    target_user: String,
    persist_dir: String,
    temp_path: String,
    home_path: String,
}

impl Params {
    fn new(args: Args, temp_path: String) -> Result<Self> {
        Ok(Self {
            target_hostname: args.target_hostname,
            target_destination: args.target_destination,
            target_user: args.target_user,
            persist_dir: "/persist".to_string(),
            temp_path,
            home_path: dirs2::home_dir()
                .context("Error: No home directory find")?
                .to_str()
                .context("Error: Home directory parsing failed")?
                .to_string(),
        })
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let temp_dir = helpers::creat_tmp_dir()?;

    let params = Params::new(
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
        nixos_anywhere::setup(&params)?;
    } else {
        tracing::warn!("Go out of here ! Grrr");
    }

    Ok(())

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
}

// fn ssh_connection(params: Params) -> Result<()> {
//     let tcp = TcpStream::connect(format!("{}:{}", params.target_destination, params.ssh_port))?;
//     let mut sess = Session::new()?;
//     sess.set_tcp_stream(tcp);
//     sess.handshake()?;

//     // Authenticate with private key
//     sess.userauth_pubkey_file(&params.target_user, None, Path::new(&params.ssh_key), None)?;

//     if sess.authenticated() {
//         println!("SSH connection is established");
//         Ok(())
//     } else {
//         Err(anyhow!("SSH connection failed"))
//     }
// }

// fn scp_upload(sess: &Session, local_path: &str, remote_path: &str) -> Result<()> {
//     // let mut local_file = File::open(local_path)?;
//     // let metadata = local_file.metadata()?;
//     // let mut remote_file = sess.scp_send(Path::new(remote_path), 0o644, metadata.len(), None)?;
//     // std::io::copy(&mut local_file, &mut remote_file)?;
//     Ok(())
// }
