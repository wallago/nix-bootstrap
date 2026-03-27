use std::{
    fs::{self, File, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::PathBuf,
};

use anyhow::{Context, Result};
use tracing::{info, warn};

pub struct Info {
    known_hosts_path: PathBuf,
}

impl Info {
    pub fn new(known_hosts_path: PathBuf) -> Self {
        Self { known_hosts_path }
    }

    pub fn update_knowing_hosts(&self, destination: &str, port: &str, pk: &str) -> Result<bool> {
        info!("🔁 Update ssh knowing host");
        let known_lines: Vec<String> = BufReader::new(File::open(&self.known_hosts_path)?)
            .lines()
            .collect::<Result<_, _>>()?;
        let full_entry = format!("[{}]:{} {}", destination, port, pk);
        let host_prefix = format!("[{}]:{}", destination, port);

        if known_lines.iter().any(|line| line == &full_entry) {
            warn!("❗ Remote host is already known");
            return Ok(false);
        }

        if known_lines.iter().any(|line| line.contains(&host_prefix)) {
            info!("🔸 Remote host key has been updated in knowing hosts");
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

            fs::write(&self.known_hosts_path, updated_lines.join("\n") + "\n")?;
        } else {
            info!("🔸 Remote host key has been add in knowing hosts");

            let mut file = OpenOptions::new()
                .append(true)
                .open(&self.known_hosts_path)
                .with_context(|| format!("Failed to open {}", self.known_hosts_path.display()))?;
            writeln!(file, "{}", full_entry)?;
        }

        Ok(true)
    }
}
