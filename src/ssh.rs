use std::{
    fs::File,
    io::{self, Read},
    net::TcpStream,
    path::Path,
};

use anyhow::{Context, Result, anyhow};
use ssh_key::PublicKey;
use ssh2::{Session, Sftp};

pub struct SSH {
    session: Session,
    pub pub_key: String,
    pub port: String,
    destination: String,
    user: String,
    password: Option<String>,
    key_path: Option<String>,
}

impl SSH {
    pub fn new(
        port: String,
        destination: String,
        password: Option<String>,
        key_path: Option<String>,
        user: String,
    ) -> Result<Self> {
        let (session, pub_key) =
            Self::connect_and_authenticate(&port, &destination, &password, &key_path, &user)?;

        Ok(Self {
            session,
            pub_key,
            port,
            destination,
            user,
            password,
            key_path,
        })
    }

    fn connect_and_authenticate(
        port: &str,
        destination: &str,
        password: &Option<String>,
        key_path: &Option<String>,
        user: &str,
    ) -> Result<(Session, String)> {
        let tcp = TcpStream::connect(format!("{}:{}", destination, port))?;
        let mut sess = Session::new().context("Error: SSH session creation failed")?;
        sess.set_tcp_stream(tcp);
        sess.handshake().context("Error: SSH Handshake failed")?;

        let (pub_key_bytes, _) = sess
            .host_key()
            .ok_or(anyhow!("Error: No remote SSH key found"))?;
        let pub_key = PublicKey::from_bytes(pub_key_bytes)
            .context("Error: Host public key parsing from bytes failed")?
            .to_openssh()
            .context("Error: Host public key parsing to open ssh format failed")?;

        match key_path {
            Some(kp) => {
                sess.userauth_pubkey_file(user, None, Path::new(kp), None)
                    .context("Error: SSH authentication failed by key")?;
            }
            None => match password {
                Some(pw) => {
                    sess.userauth_password(user, pw)
                        .context("Error: SSH authentication failed by password")?;
                }
                None => return Err(anyhow!("Error: No SSH key or password was given")),
            },
        }

        if !sess.authenticated() {
            return Err(anyhow!("SSH connection authentication failed"));
        }

        Ok((sess, pub_key))
    }

    pub fn reconnect(&mut self) -> Result<()> {
        tracing::info!(
            "Reconnecting SSH session to {}:{}",
            self.destination,
            self.port
        );
        let (new_session, new_pub_key) = Self::connect_and_authenticate(
            &self.port,
            &self.destination,
            &self.password,
            &self.key_path,
            &self.user,
        )?;
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
        if Path::new(remote_path).is_file() {
            let (mut remote_file, _) = self.session.scp_recv(Path::new(remote_path))?;
            let mut contents = Vec::new();
            remote_file.read_to_end(&mut contents)?;
            remote_file.send_eof()?;
            remote_file.wait_eof()?;
            remote_file.close()?;
            remote_file.wait_close()?;
            Ok(contents)
        } else {
            return Err(anyhow!("Error: Remote path is not file: {:?}", remote_path));
        }
    }

    pub fn upload_dir(&self, sftp: &Sftp, local_path: &str, remote_path: &str) -> Result<()> {
        if Path::new(local_path).is_dir() {
            for entry in std::fs::read_dir(local_path)? {
                let entry = entry?;
                let path = entry.path();
                let remote_path = Path::new(remote_path).join(entry.file_name());
                if path.is_dir() {
                    sftp.mkdir(&remote_path, 0o755)?; // Ignore error if exists
                    self.upload_dir(
                        sftp,
                        &path
                            .to_str()
                            .ok_or(anyhow!("Error: Parsing {:?} into string failed", path))?
                            .to_string(),
                        &remote_path
                            .to_str()
                            .ok_or(anyhow!(
                                "Error: Parsing {:?} into string failed",
                                remote_path
                            ))?
                            .to_string(),
                    )?;
                } else if path.is_file() {
                    let mut local_file = File::open(&path)?;
                    let mut remote_file = sftp.create(&remote_path)?;
                    io::copy(&mut local_file, &mut remote_file)?;
                }
            }
        } else {
            return Err(anyhow!(
                "Error: Local path is not a directory: {:?}",
                local_path
            ));
        }
        Ok(())
    }

    pub fn get_sftp(&self) -> Result<Sftp> {
        Ok(self.session.sftp()?)
    }
}
