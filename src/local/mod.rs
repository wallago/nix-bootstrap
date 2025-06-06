use std::{
    fs::{self, File, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::PathBuf,
};

use anyhow::{Context, Result, anyhow};
use dialoguer::{Confirm, Select, theme::ColorfulTheme};
use git2::Repository;
use tempfile::TempDir;
use tracing::{info, warn};

use crate::{
    helpers::{self},
    remote,
};

struct Repo {
    is_starter: bool,
    git: Repository,
    path: PathBuf,
    tmp_dir: TempDir,
}

pub struct Host {
    repo: Option<Repo>,
    pub ssh_pk_path: PathBuf,
    pub ssh_sk_path: PathBuf,
    pub ssh_known_hosts: PathBuf,
    ssh_pk: String,
    host: Option<String>,
}

impl Host {
    pub fn new() -> Result<Self> {
        let home_dir =
            dirs2::home_dir().ok_or_else(|| anyhow!("Could not find local home directory"))?;
        let ssh_pk = fs::read_to_string(home_dir.join(".ssh/id_ed25519.pub"))?;

        Ok(Self {
            repo: None,
            ssh_pk_path: home_dir.join(".ssh/id_ed25519.pub"),
            ssh_sk_path: home_dir.join(".ssh/id_ed25519"),
            ssh_known_hosts: home_dir.join(".ssh/known_hosts"),
            ssh_pk,
            host: None,
        })
    }

    fn get_host(&self) -> Result<&str> {
        Ok(self
            .host
            .as_ref()
            .ok_or_else(|| anyhow!("Host has not been set"))?)
    }

    pub fn update_ssh_knowing_hosts(
        &self,
        destination: &str,
        port: &str,
        pk: &str,
    ) -> Result<bool> {
        info!("📝 Update ssh knowing host");
        let known_lines: Vec<String> = BufReader::new(File::open(&self.ssh_known_hosts)?)
            .lines()
            .collect::<Result<_, _>>()?;
        let full_entry = format!("[{}]:{} {}", destination, port, pk);
        let host_prefix = format!("[{}]:{}", destination, port);

        if known_lines.iter().any(|line| line == &full_entry) {
            warn!("❗ Remote host is already known");
            return Ok(false);
        }

        if known_lines.iter().any(|line| line.contains(&host_prefix)) {
            info!("Remote host key updated");
            let updated_lines: Vec<String> = known_lines
                .into_iter()
                .map(|line| {
                    if line.contains(&host_prefix) {
                        full_entry.clone()
                    } else {
                        line
                    }
                })
                .collect();

            fs::write(&self.ssh_known_hosts, updated_lines.join("\n") + "\n")?;
        } else {
            info!("Remote host has been add");

            let mut file = OpenOptions::new()
                .append(true)
                .open(&self.ssh_known_hosts)
                .with_context(|| format!("Failed to open {}", self.ssh_known_hosts.display()))?;
            writeln!(file, "{}", full_entry)?;
        }

        Ok(true)
    }

    pub fn git_clone_nix_stater_config(&mut self) -> Result<()> {
        info!("📂 Clone nix-stater-config git repository");
        let (repo, tmp_dir) = helpers::git::git_clone_repository("nix-stater-config")?;
        let repo_dir = repo.path().to_path_buf();
        let repo_path = repo_dir
            .parent()
            .context("Could not get parent path of cloned git repository")?;
        self.repo = Some(Repo {
            is_starter: true,
            git: repo,
            path: repo_path.to_path_buf(),
            tmp_dir,
        });
        Ok(info!(
            "📂 Git repository is available at {}",
            repo_path.display()
        ))
    }

    pub fn git_clone_nix_config(&mut self) -> Result<()> {
        info!("📂 Clone nix-config git repository ");
        let (repo, tmp_dir) = helpers::git::git_clone_repository("nix-config")?;
        let repo_dir = repo.path().to_path_buf();
        let repo_path = repo_dir
            .parent()
            .context("Could not get parent path of cloned git repository")?;
        self.repo = Some(Repo {
            is_starter: false,
            git: repo,
            path: repo_path.to_path_buf(),
            tmp_dir,
        });
        Ok(info!(
            "📂 Git repository is available at {}",
            repo_path.display()
        ))
    }

    fn get_repo(&self) -> Result<&Repo> {
        Ok(self
            .repo
            .as_ref()
            .ok_or_else(|| anyhow!("Git repo not seems to be cloned"))?)
    }

    pub fn deploy_nix_stater_config(&self, remote: &remote::Host) -> Result<bool> {
        if !Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("📣 Do you want to run nixos-anywhere?")
            .interact()?
        {
            warn!("❗ Skipping deployments via nixos-anywhere");
            return Ok(false);
        }

        info!("🚀 Deploying nix-stater-config via nixos-anywhere");
        helpers::command::run_command(&format!(
            "nix run github:nix-community/nixos-anywhere -- --ssh-port {} --flake {}#{} {}@{}",
            remote.port,
            self.get_repo()?.path.display(),
            "plankton",
            remote.user,
            remote.destination,
        ))?;

        Ok(true)
    }

    pub fn deploy_nix_config(&self, remote: &remote::Host) -> Result<bool> {
        if !Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("📣 Do you want to run nixos-rebuild?")
            .interact()?
        {
            warn!("❗ Skipping deployments via nixos-rebuild");
            return Ok(false);
        }

        info!("🚀 Deploying nix-config via nixos-rebuild");
        helpers::command::run_command(&format!(
            "NIX_SSHOPTS=\"-p {}\" nixos-rebuild switch --flake {}#{} --build-host {}@{} --target-host {}@{} --use-remote-sudo",
            remote.port,
            self.get_repo()?.path.display(),
            self.get_host()?,
            remote.user,
            remote.destination,
            remote.user,
            remote.destination,
        ))?;

        Ok(true)
    }

    pub fn update_hardware_config(&self, contents: &Vec<u8>) -> Result<bool> {
        info!("📝 Update hardware config");
        let repo = self.get_repo()?;
        let hardware_config_path = match repo.is_starter {
            true => format!("{}/nixos/hardware-configuration.nix", repo.path.display()),
            false => {
                warn!("❗ Not available for nix-config (only for the stater)");
                return Ok(false);
            }
        };
        fs::write(hardware_config_path, contents)?;
        Ok(true)
    }

    pub fn update_disk_config(&self, contents: &str) -> Result<bool> {
        info!("📝 Update disk config");
        let repo = self.get_repo()?;
        let disk_config_path = match repo.is_starter {
            true => format!("{}/nixos/disk-device.txt", repo.path.display()),
            false => {
                warn!("❗ Not available for nix-config (only for the stater)");
                return Ok(false);
            }
        };
        fs::write(disk_config_path, format!("/dev/{}", contents))?;
        Ok(true)
    }

    pub fn update_ssh_public_key(&self) -> Result<bool> {
        info!("📝 Update ssh public key");
        let repo = self.get_repo()?;
        let ssh_public_key_path = match repo.is_starter {
            true => format!("{}/nixos/ssh_authorized_key.pub", repo.path.display()),
            false => {
                warn!("❗ Not available for nix-config (only for the stater)");
                return Ok(false);
            }
        };
        fs::write(ssh_public_key_path, &self.ssh_pk)?;
        Ok(true)
    }

    pub fn update_sops(&self, contents: &str) -> Result<bool> {
        info!("📝 Update SOPS");
        let repo = self.get_repo()?;
        let sops_path = repo.path.join(".sops.yaml");
        let host = self.get_host()?;
        let key_line_prefix = format!("- &{host}",);
        let new_key_line = format!("    - &{host} {}", contents);
        let new_ref_line = format!("          - *{host}");
        let file =
            File::open(&sops_path).context(format!("Opening {} failed", sops_path.display()))?;
        let mut lines: Vec<String> = BufReader::new(file)
            .lines()
            .collect::<Result<_, _>>()
            .context("Reading lines failed")?;

        let mut key_view_index: Option<usize> = None;
        let mut keys: Vec<String> = Vec::new();

        let mut ref_view_index: Option<usize> = None;
        let mut refs: Vec<String> = Vec::new();

        for (index, line) in lines.iter().enumerate() {
            if line.trim_start().starts_with(&key_line_prefix) {
                if line.trim() == new_key_line.trim() {
                    warn!("❗ Key was already into SOPS for {host}");
                    return Ok(false);
                } else {
                    info!("📝 Update key into SOPS for {host}");
                    lines[index] = new_key_line.clone();
                    fs::write(sops_path, lines.join("\n"))?;
                    return Ok(true);
                }
            }

            if line.trim() == "users: &age_keys" {
                key_view_index = Some(index);
            } else if line.trim() == "- age:" {
                ref_view_index = Some(index);
            }

            if let Some(key_index) = key_view_index {
                if key_index == index - 1 && line.trim_start().starts_with("- &") {
                    keys.push(line.trim().to_string());
                    key_view_index = Some(index);
                }
            }

            if let Some(ref_index) = ref_view_index {
                if ref_index == index - 1 && line.trim_start().starts_with("- *") {
                    refs.push(line.trim().to_string());
                    ref_view_index = Some(index);
                }
            }
        }

        let mut new_key_added = false;
        if !keys.iter().any(|key| key == new_key_line.trim()) {
            if let Some(key_index) = key_view_index {
                lines.insert(key_index + 1, new_key_line.to_string());
                new_key_added = true;
            }
        }

        let mut new_ref_added = false;
        if new_key_added && !refs.iter().any(|r#ref| r#ref == new_ref_line.trim()) {
            if let Some(ref_index) = ref_view_index {
                lines.insert(ref_index + 2, new_ref_line.to_string());
                new_ref_added = true;
            }
        }

        if new_key_added && new_ref_added {
            info!("📝 New key added into SOPS for {host}");
            fs::write(sops_path, lines.join("\n"))?;
            Ok(true)
        } else {
            Err(anyhow!("Failed to update SOPS file"))
        }
    }

    pub fn get_config_host(&mut self) -> Result<()> {
        let repo = self.get_repo()?;
        let hosts = serde_json::from_str::<Vec<String>>(
            &helpers::command::run_command_with_stdout(&format!(
                " nix eval --json {}#nixosConfigurations --apply builtins.attrNames",
                repo.path.display()
            ))?,
        )?;
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("📣 Select a target block device?")
            .items(&hosts)
            .interact()?;
        self.host = Some(
            hosts
                .get(selection)
                .ok_or_else(|| anyhow!("Selected host doesn't be found"))?
                .clone(),
        );
        Ok(())
    }
}
