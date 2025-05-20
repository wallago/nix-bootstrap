use std::{
    fs::File,
    io::{self, Read},
    net::TcpStream,
    path::Path,
};

use anyhow::{Context, Result, anyhow};
use ssh_key::PublicKey;
use ssh2::{Session, Sftp};

use crate::helpers;

pub struct SshSession {
    session: Session,
    pub pub_key: String,
    pub port: String,
    pub destination: String,
    pub user: String,
}

impl SshSession {
    pub async fn new(port: String, destination: String) -> Result<Self> {
        let (session, pub_key, current_user) =
            Self::connect_and_authenticate(None, &port, &destination).await?;
        tracing::info!(
            "Connecting SSH session to {}@{}:{}",
            current_user,
            destination,
            port
        );

        Ok(Self {
            session,
            pub_key,
            user: current_user,
            port,
            destination,
        })
    }

    async fn connect_and_authenticate(
        user: Option<&str>,
        port: &str,
        destination: &str,
    ) -> Result<(Session, String, String)> {
        let mut current_user = match user {
            Some(user) => user,
            None => "nixos",
        };

        let new_user =
            helpers::enter_input(None, &format!("Enter ssh user (default: {current_user}):"))
                .await?
                .trim()
                .to_string();
        if !new_user.is_empty() {
            current_user = &new_user
        }

        let tcp = TcpStream::connect(format!("{}:{}", destination, port))?;
        let mut sess = Session::new().context("SSH session creation failed")?;
        sess.set_tcp_stream(tcp);
        sess.handshake().context("SSH Handshake failed")?;

        let (pub_key_bytes, _) = sess.host_key().ok_or(anyhow!("No remote SSH key found"))?;
        let pub_key = PublicKey::from_bytes(pub_key_bytes)
            .context("Host public key parsing from bytes failed")?
            .to_openssh()
            .context("Host public key parsing to open ssh format failed")?;

        if !helpers::ask_yes_no(&format!("Do you want to use SSH agent to connect",)).await? {
            if !helpers::ask_yes_no(&format!("Do you want to use SSH agent to connect",)).await? {
                return Err(anyhow!("No valide SSH authentication method was selected"));
            } else {
                let mut password = helpers::enter_input(None, "Enter ssh password:")
                    .await?
                    .trim()
                    .to_string();
                sess.userauth_password(current_user, &password)
                    .context("SSH authentication failed by password")?;
            }
        } else {
            sess.userauth_agent(current_user)
                .context("SSH authentication failed by agent")?;
        }

        if !sess.authenticated() {
            return Err(anyhow!("SSH connection authentication failed"));
        }

        Ok((sess, pub_key, current_user.to_string()))
    }

    pub async fn reconnect(&mut self) -> Result<()> {
        let (new_session, new_pub_key, current_user) =
            Self::connect_and_authenticate(Some(&self.user), &self.port, &self.destination).await?;
        self.user = current_user;
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
