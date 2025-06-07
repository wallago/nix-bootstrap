use std::{
    fs::{self, File},
    io::{BufRead, BufReader},
};

use anyhow::{Context, Result, anyhow};
use tracing::{info, warn};

use crate::helpers::{self};

impl super::Host {
    pub fn update_hardware_config(&self, contents: &Vec<u8>) -> Result<()> {
        info!("🔁 Update hardware config");
        let repo = self.get_repo()?;
        let hardware_config_path = format!(
            "{}/hosts/{}/hardware-configuration.nix",
            repo.path.display(),
            repo.host
        );

        fs::write(hardware_config_path, contents)?;
        Ok(())
    }

    pub fn update_disk_config(&self, contents: &str) -> Result<bool> {
        info!("🔁 Update disk config");
        let repo = self.get_repo()?;
        let host = repo.host.clone();
        let host_path = repo.path.join(format!("hosts/{host}/default.nix"));
        let line_prefix = "disk.path = \"";
        let new_line = format!("  disk.path = \"/dev/{}\";", contents);
        let file =
            File::open(&host_path).context(format!("Opening {} failed", host_path.display()))?;
        let mut lines: Vec<String> = BufReader::new(file)
            .lines()
            .collect::<Result<_, _>>()
            .context("Reading lines failed")?;

        for (index, line) in lines.iter().enumerate() {
            if line.trim_start().starts_with(&line_prefix) {
                if line.trim() == new_line.trim() {
                    warn!("❗ Disk device was already set for {host}");
                    return Ok(false);
                } else {
                    info!("🔸 Update disk device for {host}");
                    lines[index] = new_line.clone();
                    fs::write(host_path, lines.join("\n"))?;
                    return Ok(true);
                }
            }
        }
        Err(anyhow!("Disk device has not been find"))
    }

    pub fn update_sops(&self, contents: &str) -> Result<bool> {
        info!("🔁 Update SOPS");
        let repo = self.get_repo()?;
        let sops_path = repo.path.join(".sops.yaml");
        let host: &str = repo.host.as_ref();
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
                    info!("🔸 Update key into SOPS for {host}");
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
            info!("🔁 New key added into SOPS for {host}");
            fs::write(sops_path, lines.join("\n"))?;
            Ok(true)
        } else {
            Err(anyhow!("Failed to update SOPS file"))
        }
    }

    pub fn update_encrypt_file_keys(&self, age_pk: &str) -> Result<()> {
        info!("🔁 Update encryt file with remote age key");
        let repo = self.get_repo()?;
        let encryt_file_path = format!("{}/nixos/common/secrets.yaml", repo.path.display());
        let contents = helpers::command::run_with_stdout(&format!(
            "sops -r --add-age {} {}",
            age_pk, encryt_file_path
        ))?;
        Ok(fs::write(encryt_file_path, contents)?)
    }
}
