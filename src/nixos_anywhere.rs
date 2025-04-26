use anyhow::{Context, Result, anyhow};
use rand::rngs::OsRng;
use ssh_key::{LineEnding, PrivateKey, PublicKey};
use std::{
    fs::{self, OpenOptions},
    io::{BufRead, BufReader, Write},
    os::unix::fs::PermissionsExt,
    path::Path,
};

use crate::{Params, helpers};

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

    let sess = helpers::ssh_handshake(params)?;
    let (host_key, _) = sess
        .host_key()
        .ok_or(anyhow!("Error: No remote SSH key found"))?;

    let host_pub_key = PublicKey::from_bytes(host_key)
        .context("Error: Host public key parsing from bytes failed")?
        .to_openssh()
        .context("Error: Host public key parsing to open ssh fromat failed")?;

    let host_entry = format!(
        "[{}]:{} {}",
        params.target_destination, params.ssh_port, host_pub_key
    );
    let known_hosts_content = fs::read_to_string(&home_ssh_path)
        .context(format!("Error: Failed to read {}", home_ssh_path))?;
    if !known_hosts_content.contains(&host_entry) {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&home_ssh_path)
            .context(format!(
                "Error: Failed to open or apppend {}",
                home_ssh_path
            ))?;
        writeln!(file, "{}", host_entry).context(format!(
            "Error: Failed to add line {} into {}",
            host_pub_key, home_ssh_path
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
        let sess = helpers::ssh_handshake(params)?;
        helpers::ssh_auth(params, sess)?;

        // should try to connect to it and find the way to make sudo command lol
        //     $ssh_root_cmd "nixos-generate-config --no-filesystems --root /mnt"
        //     $scp_cmd root@"$target_destination":/mnt/etc/nixos/hardware-configuration.nix \
        //       "${git_root}"/hosts/nixos/"$target_hostname"/hardware-configuration.nix

        params.generated_hardware_config = true;
    } else {
        tracing::warn!("Go out of here ! Grrr");
    };
    Ok(())
}
