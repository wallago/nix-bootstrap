use std::{
    fs::{self},
    path::PathBuf,
};

use crate::config::Config;
use crate::ssh::SshSession;
use anyhow::Result;

// pub fn generate_age_key(config: &Config, ssh: &SshSession) -> Result<()> {
//     tracing::info!(
//         "Generating an age key based on the ssh key for {}@{}",
//         ssh.user,
//         ssh.destination
//     );

//     let host_age_key = ssh_to_age::convert::ssh_public_key_to_age(&ssh.pub_key)?;
//     tracing::debug!("ssh target public key: {}", ssh.pub_key);
//     tracing::debug!("ssh target public key to age: {}", host_age_key.to_string());

//     tracing::info!("Updating .sops.yaml");
//     config_sops_update_age_key(
//         config.path.clone().unwrap(),
//         &format!("{}_{}", ssh.user, config.hostname,),
//         &host_age_key.to_string(),
//     )?;

//     tracing::info!("Updating target ssh public key");
//     write_target_ssh_public_key(config.path.clone().unwrap(), &config.hostname, &ssh.pub_key)
// }

// fn config_sops_update_age_key(config_path: PathBuf, key_name: &str, key_value: &str) -> Result<()> {
//     let sops_path = format!("{}/.sops.yaml", config_path.display());
//     let mut lines = fs::read_to_string(&sops_path)?
//         .lines()
//         .map(|s| s.to_string())
//         .collect::<Vec<_>>();

//     if !replace_existing_sops_age_key(&sops_path, key_name, key_value, &mut lines)? {
//         insert_new_sops_age_key(&sops_path, key_name, key_value, &mut lines)?;
//     }

//     Ok(())
// }

// fn replace_existing_sops_age_key(
//     sops_path: &str,
//     key_name: &str,
//     key_value: &str,
//     lines: &mut Vec<String>,
// ) -> Result<bool> {
//     let key_line_prefix = format!("- &{key_name}");
//     let new_key_line = format!("    - &{key_name} {key_value}");

//     let existing_key_line = lines
//         .iter()
//         .enumerate()
//         .find(|(_, line)| line.trim_start().starts_with(&key_line_prefix));

//     if let Some((line_index, line)) = existing_key_line {
//         if line.trim() != new_key_line.trim() {
//             lines[line_index] = new_key_line.clone();

//             tracing::info!("Update key added into SOPS for {key_name}");
//             fs::write(sops_path, lines.join("\n"))?;
//             Ok(true)
//         } else {
//             Ok(false)
//         }
//     } else {
//         Ok(false)
//     }
// }

// fn insert_new_sops_age_key(
//     sops_path: &str,
//     key_name: &str,
//     key_value: &str,
//     lines: &mut Vec<String>,
// ) -> Result<()> {
//     let new_key_line = format!("    - &{key_name} {key_value}");
//     let new_ref_line = format!("          - *{key_name}");

//     let mut key_view_index: Option<usize> = None;
//     let mut keys: Vec<String> = Vec::new();

//     let mut ref_view_index: Option<usize> = None;
//     let mut refs: Vec<String> = Vec::new();

//     for i in 0..lines.len() {
//         if lines[i].trim() == "users: &age_keys" {
//             key_view_index = Some(i);
//         } else if lines[i].trim() == "- age:" {
//             ref_view_index = Some(i);
//         }

//         if let Some(key_index) = key_view_index {
//             if key_index == i - 1 && lines[i].trim_start().starts_with("- &") {
//                 keys.push(lines[i].trim().to_string());
//                 key_view_index = Some(i);
//             }
//         }

//         if let Some(ref_index) = ref_view_index {
//             if ref_index == i - 1 && lines[i].trim_start().starts_with("- *") {
//                 refs.push(lines[i].trim().to_string());
//                 ref_view_index = Some(i);
//             }
//         }
//     }

//     let mut new_key_added = false;
//     if !keys.iter().any(|key| key == new_key_line.trim()) {
//         if let Some(key_index) = key_view_index {
//             lines.insert(key_index + 1, new_key_line.to_string());
//             new_key_added = true;
//         }
//     }

//     let mut new_ref_added = false;
//     if new_key_added && !refs.iter().any(|r#ref| r#ref == new_ref_line.trim()) {
//         if let Some(ref_index) = ref_view_index {
//             lines.insert(ref_index + 2, new_ref_line.to_string());
//             new_ref_added = true;
//         }
//     }

//     if new_key_added && new_ref_added {
//         tracing::info!("New key added into SOPS for {key_name}");
//         fs::write(sops_path, lines.join("\n"))?;
//     } else {
//         tracing::warn!("Key was already into SOPS for {key_name}");
//     }

//     Ok(())
// }

// fn write_target_ssh_public_key(config_path: PathBuf, host: &str, ssh_pk: &str) -> Result<()> {
//     let ssh_pk_path = format!(
//         "{}/hosts/{}/ssh_host_ed25519_key.pub",
//         config_path.display(),
//         host
//     );
//     Ok(fs::write(ssh_pk_path, ssh_pk)?)
// }

pub fn update_target_ssh_authorized_key(config_path: PathBuf) -> Result<()> {
    tracing::info!("Update target authorized ssh key with host key");
    let ssh_host_pk = get_host_ssh_public_key()?;

    tracing::debug!("ssh target public key: {}", ssh_host_pk);
    write_host_ssh_public_key(config_path, &ssh_host_pk)
}

fn get_host_ssh_public_key() -> Result<String> {
    let ssh_pk_path = "/etc/ssh/ssh_host_ed25519_key.pub";
    let ssh_pk = fs::read_to_string(ssh_pk_path)?;
    Ok(ssh_pk.trim().to_string())
}

fn write_host_ssh_public_key(config_path: PathBuf, ssh_pk: &str) -> Result<()> {
    let ssh_pk_path = format!("{}/nixos/ssh_host_ed25519_key.pub", config_path.display(),);
    Ok(fs::write(ssh_pk_path, ssh_pk)?)
}
