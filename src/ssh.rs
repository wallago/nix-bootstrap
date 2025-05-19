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
    pub async fn new(user: String, port: String, destination: String) -> Result<Self> {
        tracing::info!(
            "Connecting SSH session to {}@{}:{}",
            user,
            destination,
            port
        );

        let (session, pub_key) = Self::connect_and_authenticate(&user, &port, &destination).await?;
        Ok(Self {
            session,
            pub_key,
            user,
            port,
            destination,
        })
    }

    async fn connect_and_authenticate(
        user: &str,
        port: &str,
        destination: &str,
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

        if !helpers::ask_yes_no(&format!("Do you want to use SSH agent to connect",)).await? {
            if !helpers::ask_yes_no(&format!("Do you want to use SSH agent to connect",)).await? {
                return Err(anyhow!("No valide SSH authentication method was selected"));
            } else {
                let mut password = helpers::enter_input(None, "Enter ssh password:")
                    .await?
                    .trim()
                    .to_string();
                sess.userauth_password(user, &password)
                    .context("SSH authentication failed by password")?;
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

    pub async fn reconnect(&mut self, user: Option<String>) -> Result<()> {
        if let Some(user) = user {
            self.user = user
        }
        tracing::info!(
            "Reconnecting SSH session to {}@{}:{}",
            self.user,
            self.destination,
            self.port
        );
        let (new_session, new_pub_key) =
            Self::connect_and_authenticate(&self.user, &self.port, &self.destination).await?;
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

//     pub fn upload_dir(&self, sftp: &Sftp, local_path: &str, remote_path: &str) -> Result<()> {
//         if Path::new(local_path).is_dir() {
//             for entry in std::fs::read_dir(local_path)? {
//                 let entry = entry?;
//                 let path = entry.path();
//                 let remote_path = Path::new(remote_path).join(entry.file_name());
//                 if path.is_dir() {
//                     sftp.mkdir(&remote_path, 0o755)?; // Ignore error if exists
//                     self.upload_dir(
//                         sftp,
//                         &path
//                             .to_str()
//                             .ok_or(anyhow!("Error: Parsing {:?} into string failed", path))?
//                             .to_string(),
//                         &remote_path
//                             .to_str()
//                             .ok_or(anyhow!(
//                                 "Error: Parsing {:?} into string failed",
//                                 remote_path
//                             ))?
//                             .to_string(),
//                     )?;
//                 } else if path.is_file() {
//                     let mut local_file = File::open(&path)?;
//                     let mut remote_file = sftp.create(&remote_path)?;
//                     io::copy(&mut local_file, &mut remote_file)?;
//                 }
//             }
//         } else {
//             return Err(anyhow!(
//                 "Error: Local path is not a directory: {:?}",
//                 local_path
//             ));
//         }
//         Ok(())
//     }

//     pub fn get_sftp(&self) -> Result<Sftp> {
//         match &self.session {
//             Some(sess) => Ok(sess.sftp()?),
//             None => Err(anyhow!("SSH session is not connected")),
//         }
//     }
// }
