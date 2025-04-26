use anyhow::{Context, Result};
use rand::rngs::OsRng;
use ssh_key::{LineEnding, PrivateKey};
use std::{
    fs,
    io::{BufRead, BufReader, Write},
    os::unix::fs::PermissionsExt,
    path::Path,
};

use crate::Params;

// Setup minimal environment for nixos-anywhere and run it
pub fn setup(params: &Params) -> Result<()> {
    remove_known_hosts_entries(params)?;

    tracing::info!(
        "Installing NixOS on remote host {} at {}",
        params.target_hostname,
        params.target_destination
    );

    ssh_key_generation(params)?;

    Ok(())
}

fn remove_known_hosts_entries(params: &Params) -> Result<()> {
    tracing::info!("Wiping known_hosts of {}", params.target_destination);
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
        "Preparing a new ssh_host_ed25519_key pair for {}",
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
        .context("Error: Failed to generate private key")?;
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

    Ok(())
}
