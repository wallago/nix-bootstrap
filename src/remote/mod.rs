use std::{
    io::Read,
    net::{TcpStream, ToSocketAddrs},
};

use anyhow::{Context, Result, anyhow, bail};
use dialoguer::{Confirm, Input, Password, theme::ColorfulTheme};
use ssh_key::PublicKey;
use ssh2::Session;
use tracing::{info, warn};

use crate::local;

#[derive(Default)]
struct Config {
    disk_device: Option<String>,
    pub hardware_file: Option<Vec<u8>>,
    age_pk: Option<String>,
    host: Option<String>,
}

pub struct Host {
    destination: String,
    user: String,
    port: String,
    ssh: Session,
    ssh_pk: String,
    pub config: Config,
}

impl Host {
    pub fn new(destination: String, port: u32, local: &local::Host) -> Result<Self> {
        let port = port.to_string();
        let (ssh, ssh_pk, user) = Self::connect(&destination, &port, local)?;
        Ok(Self {
            user,
            destination,
            port,
            ssh,
            ssh_pk,
            config: Config::default(),
        })
    }

    fn connect(
        destination: &str,
        port: &str,
        local: &local::Host,
    ) -> Result<(Session, String, String)> {
        let addr = format!("{destination}:{port}");
        info!("🔑 Try to connect (via ssh) to {addr}");
        let socket_addr = addr
            .to_socket_addrs()?
            .next()
            .ok_or_else(|| anyhow!("Could not resolve address: {addr}"))?;
        let tcp =
            TcpStream::connect(socket_addr).context(anyhow!("Failed to connect to {addr}"))?;
        let mut sess = Session::new().context("Session (ssh) creation failed")?;
        sess.set_tcp_stream(tcp);
        sess.handshake().context("Handshake (ssh) failed")?;
        let (pk_bytes, _) = sess
            .host_key()
            .ok_or(anyhow!("No public key (ssh) found"))?;
        let pk = PublicKey::from_bytes(pk_bytes)
            .context("Host public key parsing from bytes failed")?
            .to_openssh()
            .context("Host public key conversion to OpenSSH format failed")?;

        let user = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter ssh user")
            .default("nixos".to_string())
            .allow_empty(false)
            .show_default(true)
            .interact_text()?;

        if Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Do you want to use ssh agent to connect?")
            .interact()?
        {
            info!("Authentication (ssh) by agent");
            sess.userauth_agent(&user)
                .context("Authentication (ssh) failed by agent")?;
        } else {
            info!("Authenticating via password or public key fallback");
            let password = Password::with_theme(&ColorfulTheme::default())
                .with_prompt("Enter password (ssh) (leave blank to use default public key)")
                .allow_empty_password(true)
                .interact()?;
            if password.is_empty() {
                sess.userauth_pubkey_file(
                    &user,
                    Some(&local.ssh_pk_path),
                    &local.ssh_sk_path,
                    None,
                )
                .context("Authentication (ssh) failed by public key")?;
            } else {
                sess.userauth_password(&user, &password)
                    .context("Authentication (ssh) failed by password")?;
            }
        }

        if !sess.authenticated() {
            return Err(anyhow!("Authentication (ssh) failed"));
        }

        info!("🔑 Remote host connected (via ssh) to {addr}");
        Ok((sess, pk, user.to_string()))
    }

    fn run_command(&self, cmd: &str) -> Result<String> {
        let mut channel = self.ssh.channel_session().unwrap();
        channel.exec(cmd)?;
        let mut stdout = String::new();
        channel.read_to_string(&mut stdout)?;
        let mut stderr = String::new();
        channel.stderr().read_to_string(&mut stderr)?;
        channel.wait_close()?;
        let status = channel.exit_status()?;
        if status != 0 {
            bail!(
                "Command (ssh) fail with exit status ({}) and stderr: \n{}",
                status,
                stderr
            )
        }
        Ok(stdout)
    }

    fn download_file(&self, remote_path: &str) -> Result<Vec<u8>> {
        let (mut remote_file, _) = self.ssh.scp_recv(Path::new(remote_path))?;
        let mut contents = Vec::new();
        remote_file.read_to_end(&mut contents)?;
        remote_file.send_eof()?;
        remote_file.wait_eof()?;
        remote_file.close()?;
        remote_file.wait_close()?;
        Ok(contents)
    }

    pub fn get_hardware_config(&self) -> Result<bool> {
        tracing::info!("🔧 Get remote hardware configuration");
        if !Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(format!(
                "Do you want to get hardware-configuration.nix on {}@{}",
                self.user, self.destination
            ))
            .interact()?
        {
            warn!("Skipping hardware-configuration part");
            return Ok(false);
        }

        self.run_command("nixos-generate-config --no-filesystems --root /tmp")?;
        self.config.hardware_file =
            Some(self.download_file("/tmp/etc/nixos/hardware-configuration.nix")?);
        Ok(true)
    }
}
