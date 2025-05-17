use anyhow::{Context, Result, anyhow};
use rand::rngs::OsRng;
use ssh_key::{LineEnding, PrivateKey};
use std::{
    fs::{self, OpenOptions},
    io::{BufRead, BufReader, Write},
    os::unix::fs::PermissionsExt,
    path::Path,
};

use crate::{
    Params,
    helpers::{self, add_ssh_host_fingerprint},
};

pub async fn setup(params: &mut Params) -> Result<()> {
    if !helpers::ask_yes_no("Run nixos-anywhere installation ?").await? {
        return Err(anyhow!("Won't run nixos-anywhere installation"));
    }

    tracing::info!(
        "Installing NixOS on remote host {} at {}",
        params.target_hostname,
        params.target_destination
    );

    remove_target_ssh_fingerprint(params)?;

    generate_target_ssh_key(params)?;

    if helpers::ask_yes_no("Generate a new hardware config for this host ? (optional)").await? {
        generate_hardware_config(params)?;
    }

    run_nixos_anywhere(params)?;

    if !helpers::ask_yes_no("Has your system restarted and are you ready to continue? (no exits)")
        .await?
    {
        return Err(anyhow!("nixos-anywhere seems to failed"));
    };

    params.ssh.reconnect()?;

    add_ssh_host_fingerprint(params)?;

    make_some_files_persistent_on_target(params)
}

fn remove_target_ssh_fingerprint(params: &Params) -> Result<()> {
    tracing::info!(
        "Wiping knowning hosts of {} or {}",
        params.target_hostname,
        params.target_destination
    );
    let patterns = [&params.target_hostname, &params.target_destination].to_vec();
    let ssh_know_hosts_path = format!("{}/.ssh/known_hosts", params.home_path);
    let file_in = fs::File::open(&ssh_know_hosts_path)
        .context(format!("Error: No file {}", ssh_know_hosts_path))?;
    let reader = BufReader::new(file_in);
    let lines: Vec<String> = reader
        .lines()
        .map_while(Result::ok)
        .filter(|line| !patterns.iter().any(|pat| line.contains(*pat)))
        .collect();
    let mut file_out = fs::File::create(&ssh_know_hosts_path)
        .context(format!("Error: Failed to create {}", ssh_know_hosts_path))?;
    for line in lines {
        writeln!(file_out, "{line}").context(format!(
            "Error: Failed to add line {} into {}",
            line, ssh_know_hosts_path
        ))?;
    }
    Ok(())
}

fn generate_target_ssh_key(params: &Params) -> Result<()> {
    tracing::info!(
        "Preparing a new ssh host ed25519 key pair for {}",
        params.target_hostname
    );
    let ssh_path = format!("{}/{}/etc/ssh", params.temp_path, params.persist_dir);
    let path = Path::new(&ssh_path);
    fs::create_dir_all(path).context(format!("Error: Failed to create {}", ssh_path))?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o755))
        .context(format!("Error: Failed to set permissions for {}", ssh_path))?;

    let priv_key_path = format!("{}/ssh_host_ed25519_key", ssh_path);
    let pub_key_path = format!("{}.pub", priv_key_path);

    let priv_key = PrivateKey::random(&mut OsRng, ssh_key::Algorithm::Ed25519)
        .context("Error: Failed to generate ed25519 private key")?;
    priv_key
        .write_openssh_file(Path::new(&priv_key_path), LineEnding::LF)
        .context(format!("Error: Failed to create {}", priv_key_path))?;

    let pub_key = priv_key.public_key();
    std::fs::write(
        &pub_key_path,
        format!(
            "{} {}@{}\n",
            pub_key
                .to_openssh()
                .context("Error: Failed to get public key")?,
            params.target_user,
            params.target_hostname
        ),
    )
    .context(format!(
        "Error: Failed to write public key into {}",
        pub_key_path
    ))?;
    fs::set_permissions(&priv_key_path, fs::Permissions::from_mode(0o600)).context(format!(
        "Error: Failed to set permissions for {}",
        priv_key_path
    ))?;

    add_ssh_host_fingerprint(params)?;

    Ok(())
}

fn generate_hardware_config(params: &mut Params) -> Result<()> {
    tracing::info!(
        "Generating hardware-configuration.nix on {}.",
        params.target_hostname
    );

    params
        .ssh
        .run_command("nixos-generate-config --no-filesystems --root /mnt")?;
    let contents = params
        .ssh
        .download_file("/mnt/etc/nixos/hardware-configuration.nix")?;
    let local_path = format!(
        "{}/hosts/{}/hardware-configuration.nix",
        params.config, params.target_hostname
    );
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&local_path)
        .context(format!("Error: Failed to open or apppend {}", local_path))?;
    file.write_all(&contents)?;

    params.generated_hardware_config = true;
    Ok(())
}

fn run_nixos_anywhere(params: &Params) -> Result<()> {
    tracing::info!(
        "Adding {}'s ssh host fingerprint to ~/.ssh/known_hosts",
        params.target_destination
    );

    helpers::run_command(&format!(
        "nix run github:nix-community/nixos-anywhere -- --extra-files {} --ssh-port {} --post-kexec-ssh-port {} --flake {}#{} {}@{}",
        params.temp_path,
        params.ssh.port,
        params.ssh.port,
        params.config,
        params.target_hostname,
        params.target_user,
        params.target_destination
    ))?;

    Ok(())
}

fn make_some_files_persistent_on_target(params: &Params) -> Result<()> {
    tracing::info!(
        "Adding {}'s ssh host fingerprint to ~/.ssh/known_hosts",
        params.target_destination
    );

    if Path::new(&params.persist_dir).exists() {
        params.ssh.run_command(&format!(
            "cp /etc/machine-id {}/etc/machine-id || true",
            params.persist_dir
        ))?;
        params.ssh.run_command(&format!(
            "cp -R /etc/ssh/ {}/etc/ssh/ || true",
            params.persist_dir
        ))?;
    }
    Ok(())
}
