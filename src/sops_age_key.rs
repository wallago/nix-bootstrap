use std::{fs, str::FromStr};

use anyhow::{Result, anyhow};
use serde_yaml::Value;

use crate::{Params, helpers};

pub async fn setup(params: &mut Params) -> Result<()> {
    if helpers::ask_yes_no("Generate host (ssh-based) age key ?").await? {
        tracing::info!("Generating an age key based on the new ssh_host_ed25519_key");

        let host_age_key = age::ssh::Recipient::from_str(&params.ssh.host_key)
            .map_err(|_| anyhow!("Failed to parse SSH host key into an age recipient"))?;

        tracing::info!("Updating .sops.yaml");
        sops_update_age_key(
            params,
            "hosts",
            &params.target_hostname,
            &host_age_key.to_string(),
        )?;
    } else {
        tracing::warn!("Go out of here ! Grrr");
    }

    if helpers::ask_yes_no("Generate user age key ?").await? {
        tracing::info!("Generating an age key based on the new ssh_host_ed25519_key");

        let key_name = format!("${}_${}", params.target_user, params.target_hostname);
        let user_age_key = age::x25519::Identity::generate().to_public();

        tracing::info!("Updating .sops.yaml");
        sops_update_age_key(params, "users", &key_name, &user_age_key.to_string())?;
    } else {
        tracing::warn!("Go out of here ! Grrr");
    }

    Ok(())
}

fn sops_update_age_key(
    params: &Params,
    field: &str,
    key_name: &str,
    key_value: &str,
) -> Result<()> {
    if field != "hosts" && field != "users" {
        return Err(anyhow!(
            "Error: wrong field for .sops.yaml. Must be either 'hosts' or 'users'."
        ));
    }

    let sops_path = format!("{}/.sops.yaml", params.git_dir_path);
    let contents = fs::read_to_string(&sops_path)?;
    let sops: Value = serde_yaml::from_str(&contents)?;
    let mut sops_copy = sops.clone();
    let key_list = sops_copy
        .get_mut("keys")
        .ok_or(anyhow!("Error: No 'keys' in .sops.yaml"))?
        .get_mut(field)
        .ok_or(anyhow!("Error: No '{field}' in .sops.yaml"))?
        .as_sequence_mut()
        .ok_or(anyhow!("Error: '{field}' should be a sequence"))?;

    let mut found = false;
    for entry in key_list.iter_mut() {
        if let Some(anchor) = entry.as_str() {
            if anchor.contains(&key_name) {
                *entry = Value::String(key_value.to_string());
                found = true;
                tracing::info!("Updated existing {key_name} key");
                break;
            }
        }
    }

    if !found {
        tracing::info!("Adding new {key_name} key");
        key_list.push(Value::String(key_value.to_string()));
    }
    // Convert back to YAML and inject anchors manually
    let mut out = serde_yaml::to_string(&sops)?;
    if !found {
        let needle = format!("- {}", key_value);
        let replacement = format!("- &{} {}", key_name, key_value);
        out = out.replacen(&needle, &replacement, 1);
    }

    fs::write(&sops_path, out)?;

    Ok(())
}
