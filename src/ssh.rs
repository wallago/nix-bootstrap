use std::{io::Read, net::TcpStream, path::Path};

use anyhow::{Context, Result, anyhow};
use ssh_key::PublicKey;
use ssh2::Session;

use crate::helpers;

pub struct SshSession {
    session: Session,
    pub pub_key: String,
    pub port: String,
    pub destination: String,
    pub user: String,
    pub password: Option<String>,
}

impl SshSession {
    pub async fn new(
        port: i32,
        destination: String,
        user: &str,
        password: Option<String>,
    ) -> Result<Self> {
        let (session, pub_key) =
            Self::connect_and_authenticate(&port.to_string(), &destination, user, password).await?;
        tracing::info!(
            "Connecting SSH session to {}@{}:{}",
            user,
            destination,
            port
        );

        Ok(Self {
            session,
            pub_key,
            port: port.to_string(),
            destination,
            user: user.to_string(),
            password,
        })
    }

    async fn connect_and_authenticate(
        port: &str,
        destination: &str,
        user: &str,
        password: Option<String>,
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

        if !helpers::input::ask_yes_no(&format!("Do you want to use SSH agent to connect",)).await?
        {
            if !helpers::input::ask_yes_no(&format!("Do you want to use SSH agent to connect",))
                .await?
            {
                return Err(anyhow!("No valide SSH authentication method was selected"));
            } else {
                match password {
                    Some(password) => sess
                        .userauth_password(user, &password)
                        .context("SSH authentication failed by password")?,
                    None => return Err(anyhow!("No password was set for SSH connection")),
                };
            }
        } else {
            sess.userauth_agent(user)
                .context("SSH authentication failed by agent")?;
        }

        if !sess.authenticated() {
            return Err(anyhow!("SSH connection authentication failed"));
        }

        Ok((sess, pub_key))
    }

    pub async fn reconnect(&mut self) -> Result<()> {
        let new_user = helpers::input::enter_input(
            None,
            &format!(
                "Enter ssh user if you want to change it (actual: {}):",
                self.user
            ),
        )
        .await?
        .trim()
        .to_string();
        if !new_user.is_empty() {
            self.user = new_user
        }

        let new_password = helpers::input::enter_input(
            None,
            &format!(
                "Enter ssh password if you want to change it (actual: {}):",
                self.password
            ),
        )
        .await?
        .trim()
        .to_string();
        if !new_password.is_empty() {
            self.password = Some(new_password)
        }

        let (new_session, new_pub_key) = Self::connect_and_authenticate(
            &self.port,
            &self.destination,
            &self.user,
            self.password,
        )
        .await?;
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
