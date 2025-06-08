use std::{
    fmt,
    io::Read,
    net::{TcpStream, ToSocketAddrs},
    path::Path,
    str::FromStr,
};

use anyhow::{Context, Result, anyhow, bail};
use dialoguer::{Input, Password, Select, theme::ColorfulTheme};
use ssh_key::PublicKey;
use ssh2::Session;
use tracing::info;

use crate::local;

#[derive(Debug)]
enum AuthMethod {
    Agent,
    Passwd,
}

impl fmt::Display for AuthMethod {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            AuthMethod::Agent => "agent",
            AuthMethod::Passwd => "password",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for AuthMethod {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "agent" => Ok(AuthMethod::Agent),
            "password" => Ok(AuthMethod::Passwd),
            _ => Err(format!("Invalid authentication method: {}", s)),
        }
    }
}

impl super::Host {
    pub fn reconnect(&mut self, local: &local::Host) -> Result<()> {
        let (ssh, ssh_pk, user, port) = Self::connect(&self.destination, local)?;
        self.ssh = ssh;
        self.port = port;
        self.ssh_pk = ssh_pk;
        self.user = user;
        Ok(())
    }

    pub fn connect(
        destination: &str,
        local: &local::Host,
    ) -> Result<(Session, String, String, String)> {
        info!("🔑 Try to connect (via ssh) to remote host");
        let port = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter ssh port (1-65535):")
            .default("22".to_string())
            .allow_empty(false)
            .show_default(true)
            .validate_with(|input: &String| -> Result<(), &str> {
                input
                    .parse::<u16>()
                    .map_err(|_| "Please enter a valid number between 1 and 65535")
                    .and_then(|n| {
                        if (1..=65535).contains(&n) {
                            Ok(())
                        } else {
                            Err("Port must be between 1 and 65535")
                        }
                    })
            })
            .interact_text()?;
        let addr = format!("{destination}:{port}");
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

        local.ssh.update_knowing_hosts(&destination, &port, &pk)?;

        let user = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter ssh user:")
            .default("nixos".to_string())
            .allow_empty(false)
            .show_default(true)
            .interact_text()?;

        let ssh_auth_opts = vec![AuthMethod::Agent, AuthMethod::Passwd];
        let labels: Vec<String> = ssh_auth_opts
            .iter()
            .map(|ssh_auth| ssh_auth.to_string())
            .collect();
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select an authentication method (ssh)?")
            .items(&labels)
            .interact()?;
        match ssh_auth_opts
            .get(selection)
            .ok_or_else(|| anyhow!("Authentication method (ssh) not found"))?
        {
            AuthMethod::Agent => {
                info!("🔸 Authentication (ssh) by agent");
                sess.userauth_agent(&user)
                    .context("Authentication (ssh) failed by agent")?
            }
            AuthMethod::Passwd => {
                info!("🔸 Authenticating (ssh) by password");
                let password = Password::with_theme(&ColorfulTheme::default())
                    .with_prompt("Enter password (ssh):")
                    .allow_empty_password(false)
                    .interact()?;
                sess.userauth_password(&user, &password)
                    .context("Authentication (ssh) failed by password")?;
            }
        }

        if !sess.authenticated() {
            return Err(anyhow!("Authentication (ssh) failed"));
        }

        info!("🔸 Remote host connected (via ssh) to {addr}");
        Ok((sess, pk, user.to_string(), port))
    }

    pub fn run_command(&self, cmd: &str) -> Result<String> {
        let mut channel = self
            .ssh
            .channel_session()
            .context("Failed to establish a new session-based channel (ssh)")?;
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

    pub fn download_file(&self, remote_path: &str) -> Result<Vec<u8>> {
        let (mut remote_file, _) = self.ssh.scp_recv(Path::new(remote_path))?;
        let mut contents = Vec::new();
        remote_file.read_to_end(&mut contents)?;
        remote_file.send_eof()?;
        remote_file.wait_eof()?;
        remote_file.close()?;
        remote_file.wait_close()?;
        Ok(contents)
    }
}
