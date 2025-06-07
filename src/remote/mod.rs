use std::{
    fmt,
    io::Read,
    net::{TcpStream, ToSocketAddrs},
    path::Path,
    str::FromStr,
};

use anyhow::{Context, Result, anyhow, bail};
use dialoguer::{Confirm, Input, Password, Select, theme::ColorfulTheme};
use ssh_key::PublicKey;
use ssh2::Session;
use tracing::{info, warn};

use crate::{
    helpers::{disk::DiskDevice, disk::DiskDevices},
    local,
};

#[derive(Debug)]
enum AuthMethod {
    Agent,
    Passwd,
    PublicKey,
}

impl fmt::Display for AuthMethod {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            AuthMethod::Agent => "agent",
            AuthMethod::Passwd => "password",
            AuthMethod::PublicKey => "public key",
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
            "public key" => Ok(AuthMethod::PublicKey),
            _ => Err(format!("Invalid authentication method: {}", s)),
        }
    }
}

#[derive(Default)]
pub struct Config {
    disk_device: Option<DiskDevice>,
    hardware_file: Option<Vec<u8>>,
    age_pk: Option<String>,
}

impl Config {
    pub fn get_disk_device(&self) -> Result<&DiskDevice> {
        Ok(self
            .disk_device
            .as_ref()
            .ok_or_else(|| anyhow!("Disk device has not been set"))?)
    }

    pub fn get_hardware_file(&self) -> Result<&Vec<u8>> {
        Ok(self
            .hardware_file
            .as_ref()
            .ok_or_else(|| anyhow!("Hardware file has not been set"))?)
    }

    pub fn get_age_key(&self) -> Result<&str> {
        Ok(self
            .age_pk
            .as_ref()
            .ok_or_else(|| anyhow!("Age key has not been set"))?)
    }
}

pub struct Host {
    pub destination: String,
    pub user: String,
    pub port: String,
    ssh: Session,
    pub ssh_pk: String,
    pub config: Config,
}

impl Host {
    pub fn new(destination: &str, port: &u32, local: &local::Host) -> Result<Self> {
        let port = port.to_string();
        let (ssh, ssh_pk, user) = Self::connect(&destination, &port, local)?;
        Ok(Self {
            user,
            destination: destination.to_owned(),
            port,
            ssh,
            ssh_pk,
            config: Config::default(),
        })
    }

    pub fn reconnect(&mut self, local: &local::Host) -> Result<()> {
        let (ssh, ssh_pk, user) = Self::connect(&self.destination, &self.port, local)?;
        self.ssh = ssh;
        self.ssh_pk = ssh_pk;
        self.user = user;
        Ok(())
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

        local.update_ssh_knowing_hosts(&destination, &port, &pk)?;

        let user = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter ssh user:")
            .default("nixos".to_string())
            .allow_empty(false)
            .show_default(true)
            .interact_text()?;

        let ssh_auth_opts = vec![AuthMethod::Agent, AuthMethod::Passwd, AuthMethod::PublicKey];
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
            AuthMethod::PublicKey => {
                info!("🔸 Authenticating (ssh) by public key");
                let (pk_path, sk_path) = local.get_ssh_path();
                sess.userauth_pubkey_file(&user, Some(pk_path), &sk_path, None)
                    .context("Authentication (ssh) failed by public key")?;
            }
        }

        if !sess.authenticated() {
            return Err(anyhow!("Authentication (ssh) failed"));
        }

        info!("🔸 Remote host connected (via ssh) to {addr}");
        Ok((sess, pk, user.to_string()))
    }

    fn run_command(&self, cmd: &str) -> Result<String> {
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

    pub fn get_hardware_config(&mut self) -> Result<bool> {
        if !Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(format!("Do you want to get hardware configuration?",))
            .interact()?
        {
            warn!("❗ Skipping hardware-configuration part");
            return Ok(false);
        }

        info!("🔧 Get hardware configuration");
        self.run_command("nixos-generate-config --no-filesystems --root /tmp")?;
        self.config.hardware_file =
            Some(self.download_file("/tmp/etc/nixos/hardware-configuration.nix")?);
        Ok(true)
    }

    pub fn get_disk_device(&mut self) -> Result<bool> {
        if !Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Do you want to select a disk device?")
            .interact()?
        {
            warn!("❗ Skipping disk device selection");
            return Ok(false);
        }

        let disk_devices = serde_json::from_str::<DiskDevices>(
            &self.run_command("lsblk -d -J -o NAME,SIZE,MODEL,MOUNTPOINT")?,
        )?;
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select a target block device?")
            .items(
                &disk_devices
                    .blockdevices
                    .iter()
                    .map(|disk_device| disk_device.get_info())
                    .collect::<Vec<String>>(),
            )
            .interact()?;
        let disk_device = disk_devices
            .blockdevices
            .get(selection)
            .ok_or_else(|| anyhow!("Couldn't found selected disk found"))?;
        self.config.disk_device = Some(disk_device.to_owned());
        Ok(true)
    }

    pub fn get_age_key(&mut self) -> Result<bool> {
        if !Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Do you want to get age key?")
            .interact()?
        {
            warn!("❗ Skipping age key part");
            return Ok(false);
        }

        info!("🔑 Get age key");
        self.config.age_pk = Some(ssh_to_age::convert::ssh_public_key_to_age(&self.ssh_pk)?);
        Ok(true)
    }
}
