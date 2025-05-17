use anyhow::Result;

use crate::{
    Params,
    helpers::{self},
};

pub async fn setup(params: &mut Params) -> Result<()> {
    if helpers::ask_yes_no(&format!(
        "Do you want to copy your full nix-config to {} ?",
        params.target_hostname
    ))
    .await?
    {
        params.ssh.upload_dir(
            &params.ssh.get_sftp()?,
            &params.config,
            &format!("/home/{}", params.target_hostname),
        )?;

        if helpers::ask_yes_no("Do you want to rebuild immediately ?").await? {
            tracing::info!("Rebuilding nix-config on {}", params.target_hostname);
            params.ssh.run_command(&format!(
                "cd /home/{}/nix-config && sudo nixos-rebuild --flake .#{} switch",
                params.target_hostname, params.target_hostname
            ))?;
        }
    } else {
        tracing::info!("NixOS was successfully installed !");
        tracing::info!(
            "Post-install config build instructions:
            To copy nix-config from this machine to the {}, run the following command.
            scp - 
            To rebuild, sign into {} and run the following command.
            cd nix-config
            sudo nixos-rebuild --show-trace --flake .#{} switch",
            params.target_hostname,
            params.target_hostname,
            params.target_hostname
        );
    }
    Ok(())
}
