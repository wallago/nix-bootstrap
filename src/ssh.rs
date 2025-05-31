use std::{io::Read, net::TcpStream, path::Path};

use anyhow::{Context, Result, anyhow};
use dialoguer::Confirm;
use dialoguer::Input;
use dialoguer::Password;
use dialoguer::theme::ColorfulTheme;
use ssh_key::PublicKey;
use ssh2::Session;

use crate::config;
use crate::helpers;

pub struct SshSession {
    session: Session,
    pub pub_key: String,
    pub port: String,
    pub destination: String,
    pub user: String,
}

impl SshSession {
    pub fn new(args: &config::Args) -> Result<Self> {
        let (session, pub_key) = Self::connect_and_authenticate(
            &args.ssh_port.to_string(),
            &args.ssh_dest,
            &args.ssh_user,
        )?;
        tracing::info!(
            "Connecting SSH session to {}@{}:{}",
            args.ssh_user,
            args.ssh_dest,
            args.ssh_port
        );

        Ok(Self {
            session,
            pub_key,
            port: args.ssh_port.to_string(),
            destination: args.ssh_dest.clone(),
            user: args.ssh_user.clone(),
        })
    }

    fn connect_and_authenticate(
        port: &str,
        destination: &str,
        user: &str,
    ) -> Result<(Session, String)> {
        let tcp = TcpStream::connect(format!("{}:{}", destination, port))?;
        let mut sess = Session::new().context("SSH session creation failed")?;
        sess.set_tcp_stream(tcp);
        sess.handshake().context("SSH Handshake failed")?;

        let (pub_key_bytes, _) = sess.host_key().ok_or(anyhow!("No remote SSH key found"))?;
        let pub_key = PublicKey::from_bytes(pub_key_bytes)
            .context("Host public key parsing from bytes failed")?
            .to_openssh()
            .context("Host public key parsing to open ssh format failed")?;

        if Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Do you want to use SSH agent to connect?")
            .interact()?
        {
            tracing::info!("SSH authentication by agent");
            sess.userauth_agent(user)
                .context("SSH authentication failed by agent")?;
        } else {
            tracing::info!("SSH authentication by password");

            let password = Password::with_theme(&ColorfulTheme::default())
                .with_prompt("Enter ssh password")
                .allow_empty_password(false)
                .interact()?;
            sess.userauth_password(user, &password)
                .context("SSH authentication failed by password")?;
        }

        if !sess.authenticated() {
            return Err(anyhow!("SSH connection authentication failed"));
        }

        Ok((sess, pub_key))
    }

    pub fn reconnect(&mut self) -> Result<()> {
        let new_user = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter ssh user")
            .default(self.user.clone())
            .allow_empty(false)
            .show_default(true)
            .interact_text()?;
        if !new_user.is_empty() {
            self.user = new_user
        }

        let (new_session, new_pub_key) =
            Self::connect_and_authenticate(&self.port, &self.destination, &self.user)?;
        tracing::info!(
            "Reconnecting SSH session to {}@{}:{}",
            self.user,
            self.destination,
            self.port
        );
        self.session = new_session;
        self.pub_key = new_pub_key;
        Ok(())
    }

    pub fn run_command(&self, cmd: &str) -> Result<String> {
        let mut channel = self.session.channel_session().unwrap();
        channel.exec(cmd)?;
        let mut s = String::new();
        channel.read_to_string(&mut s)?;
        channel.wait_close()?;
        Ok(s)
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
}
