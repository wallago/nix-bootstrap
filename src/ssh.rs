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

    pub fn run_command(&self, cmd: &str) -> Result<String> {
        let mut channel = self.session.channel_session().unwrap();
        channel.exec(cmd)?;
        let mut s = String::new();
        channel.read_to_string(&mut s)?;
        channel.wait_close()?;
        Ok(s)
    }

    pub fn download(&self, remote_path: &str) -> Result<Vec<u8>> {
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

    pub fn upload(&self, sftp: &Sftp, local_path: &str, remote_path: &str) -> Result<()> {
        if Path::new(local_path).is_dir() {
            for entry in std::fs::read_dir(local_path)? {
                let entry = entry?;
                let path = entry.path();
                let remote_path = Path::new(remote_path).join(entry.file_name());
                if path.is_dir() {
                    sftp.mkdir(&remote_path, 0o755)?; // Ignore error if exists
                    self.upload(
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
        } else if Path::new(local_path).is_file() {
            let mut local_file = File::open(local_path)?;
            let mut remote_file = sftp.create(Path::new(remote_path))?;
            io::copy(&mut local_file, &mut remote_file)?;
        } else {
            return Err(anyhow!(
                "Error: Local path is neither file nor directory: {:?}",
                local_path
            ));
        }
        Ok(())
    }

    pub fn get_sftp(&self) -> Result<Sftp> {
        Ok(self.session.sftp()?)
    }
}
