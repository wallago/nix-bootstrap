use std::{net::TcpStream, path::Path};

use anyhow::{Context, Result, anyhow};
use ssh_key::PublicKey;
use ssh2::Session;

pub struct SSH {
    session: Session,
    pub host_key: String,
    pub port: String,
}

impl SSH {
    pub fn new(
        port: String,
        destination: String,
        password: Option<String>,
        key_path: Option<String>,
        user: String,
    ) -> Result<Self> {
        let tcp = TcpStream::connect(format!("{}:{}", destination, port))?;
        let mut sess = Session::new().context("Error: SSH session creation failed")?;
        sess.set_tcp_stream(tcp);
        sess.handshake().context("Error: SSH Handshake failed")?;
        let (host_key, _) = sess
            .host_key()
            .ok_or(anyhow!("Error: No remote SSH key found"))?;
        let formatted_host_key = PublicKey::from_bytes(host_key)
            .context("Error: Host public key parsing from bytes failed")?
            .to_openssh()
            .context("Error: Host public key parsing to open ssh fromat failed")?;

        let ssh = Self {
            session: sess,
            host_key: formatted_host_key,
            port,
        };
        match key_path {
            Some(key_path) => ssh.auth_by_key(&user, &key_path)?,
            None => match password {
                Some(password) => ssh.auth_by_password(&user, &password)?,
                None => return Err(anyhow!("Error: No SSH key or password was given")),
            },
        }
        Ok(ssh)
    }

    fn auth_by_key(&self, user: &str, key_path: &str) -> Result<()> {
        self.session
            .userauth_pubkey_file(&user, None, Path::new(&key_path), None)
            .context("Error: SSH authentification failed by key")?;
        self.check_auth()
    }

    fn auth_by_password(&self, user: &str, password: &str) -> Result<()> {
        self.session
            .userauth_password(&user, password)
            .context("Error: SSH authentification failed by password")?;
        self.check_auth()
    }

    fn check_auth(&self) -> Result<()> {
        if self.session.authenticated() {
            tracing::info!("SSH connection is established");
            Ok(())
        } else {
            Err(anyhow!("SSH connection failed"))
        }
    }
}
