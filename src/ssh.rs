use std::{io::Read, net::TcpStream, path::Path};

use anyhow::{Context, Result, anyhow};
use dialoguer::Confirm;
use dialoguer::Input;
use dialoguer::Password;
use dialoguer::theme::ColorfulTheme;
use ssh_key::PublicKey;
use ssh2::Session;

use crate::config;

pub struct SshSession {
    session: Session,
    pub pub_key: String,
    pub port: String,
    pub destination: String,
    pub user: String,
}

impl SshSession {
    pub fn new(args: &config::Args) -> Result<Self> {
        let (session, pub_key, user) =
            Self::connect_and_authenticate(&args.ssh_port.to_string(), &args.ssh_dest)?;
        Ok(Self {
            session,
            pub_key,
            port: args.ssh_port.to_string(),
            destination: args.ssh_dest.clone(),
            user,
        })
    }

    fn connect_and_authenticate(
        port: &str,
        destination: &str,
    ) -> Result<(Session, String, String)> {
        tracing::info!("Connecting SSH session to {}:{}", destination, port);
        let tcp = TcpStream::connect(format!("{}:{}", destination, port))?;
        let mut sess = Session::new().context("SSH session creation failed")?;
        sess.set_tcp_stream(tcp);
        sess.handshake().context("SSH Handshake failed")?;

        let (pub_key_bytes, _) = sess.host_key().ok_or(anyhow!("No remote SSH key found"))?;
        let pub_key = PublicKey::from_bytes(pub_key_bytes)
            .context("Host public key parsing from bytes failed")?
            .to_openssh()
            .context("Host public key parsing to open ssh format failed")?;

        let user = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter ssh user")
            .default("nixos".to_string())
            .allow_empty(false)
            .show_default(true)
            .interact_text()?;

        if Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Do you want to use SSH agent to connect?")
            .interact()?
        {
            tracing::info!("SSH authentication by agent");
            sess.userauth_agent(&user)
                .context("SSH authentication failed by agent")?;
        } else {
            tracing::info!("SSH authentication by password (if empty public key will be used)");

            let password = Password::with_theme(&ColorfulTheme::default())
                .with_prompt("Enter ssh password")
                .allow_empty_password(true)
                .interact()?;
            if password.is_empty() {
                let home_dir = dirs2::home_dir()
                    .ok_or_else(|| anyhow::anyhow!("Host home directory isn't found"))?;
                sess.userauth_pubkey_file(
                    &user,
                    Some(home_dir.join(".ssh/id_ed25519.pub").as_path()),
                    home_dir.join(".ssh/id_ed25519").as_path(),
                    None,
                )
                .context("SSH authentication failed by password")?;
            } else {
                sess.userauth_password(&user, &password)
                    .context("SSH authentication failed by password")?;
            }
        }

        if !sess.authenticated() {
            return Err(anyhow!("SSH connection authentication failed"));
        }

        Ok((sess, pub_key, user.to_string()))
    }

    pub fn reconnect(&mut self) -> Result<()> {
        let (new_session, new_pub_key, new_user) =
            Self::connect_and_authenticate(&self.port, &self.destination)?;
        self.user = new_user;
        self.session = new_session;
        self.pub_key = new_pub_key;
        Ok(())
    }

    pub fn run_command(&self, cmd: &str) -> Result<String> {
        let mut channel = self.session.channel_session().unwrap();
        channel.exec(cmd)?;
        let mut stdout = String::new();
        channel.read_to_string(&mut stdout)?;
        let mut stderr = String::new();
        channel.stderr().read_to_string(&mut stderr)?;
        channel.wait_close()?;
        let status = channel.exit_status()?;
        if status != 0 {
            anyhow::bail!(
                "SSH command fail with exit status ({}) and stderr: \n{}",
                status,
                stderr
            )
        }

        Ok(stdout)
    }

    pub fn download_file(&self, remote_path: &str) -> Result<Vec<u8>> {
        let (mut remote_file, _) = self.session.scp_recv(Path::new(remote_path))?;
        let mut contents = Vec::new();
        remote_file.read_to_end(&mut contents)?;
        remote_file.send_eof()?;
        remote_file.wait_eof()?;
        remote_file.close()?;
        remote_file.wait_close()?;
        Ok(contents)
    }

    // pub fn upload_dir(&self, remote_path: &str) -> Result<Vec<u8>> {
    //     let (mut remote_file, _) = self.session.scp_recv(Path::new(remote_path))?;
    //     let mut contents = Vec::new();
    //     remote_file.read_to_end(&mut contents)?;
    //     remote_file.send_eof()?;
    //     remote_file.wait_eof()?;
    //     remote_file.close()?;
    //     remote_file.wait_close()?;
    //     Ok(contents)
    // }
}
