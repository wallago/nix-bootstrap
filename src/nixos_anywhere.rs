use anyhow::{Context, Result, anyhow};
use rand::rngs::OsRng;
use ssh_key::{Algorithm, LineEnding, PrivateKey};
use std::{
    fs,
    io::{BufRead, BufReader, Write},
    os::unix::fs::PermissionsExt,
    path::Path,
};

use crate::Params;

// Setup minimal environment for nixos-anywhere and run it
pub fn setup(params: &Params) -> Result<()> {
    remove_known_hosts_entries(&params)?;

    println!(
        "Installing NixOS on remote host {} at {}",
        params.target_hostname, params.target_destination
    );

    ssh_key_generation(params)?;

    Ok(())
}

fn remove_known_hosts_entries(params: &Params) -> Result<()> {
    println!("Wiping known_hosts of {}", params.target_destination);
    let patterns = [&params.target_hostname, &params.target_destination].to_vec();
    let file_in =
        fs::File::open("~/.ssh/known_hosts").context("Error: No file ~/.ssh/known_hosts")?;
    let reader = BufReader::new(file_in);
    let lines: Vec<String> = reader
        .lines()
        .filter_map(Result::ok)
        .filter(|line| !patterns.iter().any(|pat| line.contains(*pat)))
        .collect();
    let mut file_out = fs::File::create("~/.ssh/known_hosts")?;
    for line in lines {
        writeln!(file_out, "{line}")?;
    }
    Ok(())
}

fn ssh_key_generation(params: &Params) -> Result<()> {
    println!(
        "Preparing a new ssh_host_ed25519_key pair for {}",
        params.target_hostname
    );
    let ssh_path = format!("/temp/{}/etc/ssh", params.persist_dir);
    let path = Path::new(&ssh_path);
    fs::create_dir_all(&path)?;
    let mut permissions = fs::metadata(&path)?.permissions();
    permissions.set_mode(0o755);
    if !matches!(permissions.mode(), 0o755) {
        return Err(anyhow!("SSH Directory failed to set expected permissions"));
    }

    let key_path = format!("{}/ssh_host_ed25519_key", ssh_path);
    let pub_key_path = format!("{}.pub", key_path);
    let key = PrivateKey::random(&mut OsRng, Algorithm::Ed25519)?;
    key.write_openssh_file(Path::new(&key_path), LineEnding::LF)?;
    let pub_key = key.public_key();
    std::fs::write(
        pub_key_path,
        format!(
            "{} {}@{}\n",
            pub_key.to_openssh()?,
            params.target_user,
            params.target_hostname
        ),
    )?;

    let mut permissions = fs::metadata(&key_path)?.permissions();
    permissions.set_mode(0o600);
    if !matches!(permissions.mode(), 0o600) {
        return Err(anyhow!(
            "SSH private key failed to set expected permissions"
        ));
    }

    Ok(())
}
