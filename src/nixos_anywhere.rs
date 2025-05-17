use anyhow::{Context, Result};
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
    helpers::{self, is_ssh_key_exist_localy},
};

// Setup minimal environment for nixos-anywhere and run it
pub async fn setup(params: &mut Params) -> Result<()> {
    remove_known_hosts_entries(params)?;

    tracing::info!(
        "Installing NixOS on remote host {} at {}",
        params.target_hostname,
        params.target_destination
    );

    ssh_key_generation(params)?;

    generated_hardware_config(params).await?;

    if !helpers::ask_yes_no("Has your system restarted and are you ready to continue? (no exits)")
        .await?
    {
        tracing::warn!("Go out of here ! Grrr");
        return Ok(());
    };

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

fn remove_known_hosts_entries(params: &Params) -> Result<()> {
    tracing::info!("Wiping knowning hosts of {}", params.target_destination);
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

fn ssh_key_generation(params: &Params) -> Result<()> {
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
    } else {
        tracing::warn!("Already know the host fingerprint");
    }
    Ok(())
}

async fn generated_hardware_config(params: &mut Params) -> Result<()> {
    if helpers::ask_yes_no("Generate a new hardware config for this host ?").await? {
        tracing::info!(
            "Generating hardware-configuration.nix on {}.",
            params.target_hostname
        );

        params
            .ssh
            .run_command("nixos-generate-config --no-filesystems --root /mnt")?;
        let contents = params
            .ssh
            .download("/mnt/etc/nixos/hardware-configuration.nix")?;
        let local_path = format!(
            "{}/hosts/{}/hardware-configuration.nix",
            params.git_dir_path, params.target_hostname
        );
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&local_path)
            .context(format!("Error: Failed to open or apppend {}", local_path))?;
        file.write_all(&contents)?;

        params.generated_hardware_config = true;
    } else {
        tracing::warn!("Go out of here ! Grrr");
    };
    Ok(())
}
