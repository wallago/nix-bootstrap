use crate::config::Config;
use crate::helpers;
use crate::ssh::SshSession;
use anyhow::Result;

pub fn generate_age_key(config: &Config, ssh: &SshSession) -> Result<()> {
    tracing::info!(
        "Generating an age key based on the ssh key for {}@{}",
        ssh.user,
        ssh.destination
    );

    let host_age_key = ssh_to_age::convert::ssh_public_key_to_age(&ssh.pub_key)?;
    tracing::debug!("ssh pub key: {}", ssh.pub_key);
    tracing::debug!("ssh pub key to age: {}", host_age_key.to_string());

    tracing::info!("Updating .sops.yaml");
    helpers::key::sops_update_age_key(
        config.path.clone().unwrap(),
        &format!("{}_{}", ssh.user, config.hostname,),
        &host_age_key.to_string(),
    )?;

    tracing::info!("Updating ssh_host_ed25519_key.pub");
    helpers::key::ssh_update_public_key(
        config.path.clone().unwrap(),
        &config.hostname,
        &ssh.pub_key,
    )
}
