use std::fs;

use anyhow::{Context, Result, anyhow};
use tempfile::{TempDir, tempdir};
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};

use crate::Params;

pub async fn ask_yes_no(prompt: &str) -> Result<bool> {
    let mut stdout = io::stdout();
    let border = "#".repeat(prompt.len() + 11);
    stdout
        .write_all(format!("{border}\n{prompt} [y/N]: ").as_bytes())
        .await
        .context("Error: Failed to write in stdout for [y/N]")?;
    stdout
        .flush()
        .await
        .context("Error: Flushing stdout failed")?;

    let mut input = String::new();
    let mut reader = BufReader::new(io::stdin());
    reader
        .read_line(&mut input)
        .await
        .context("Error: Failed to read stdin for [y/N]")?;
    stdout
        .write_all(format!("{border}\n").as_bytes())
        .await
        .context("Error: Failed to write in stdout for [y/N]")?;

    Ok(matches!(input.trim().to_lowercase().as_str(), "y" | "yes"))
}

pub fn creat_tmp_dir() -> Result<TempDir> {
    let temp_dir = tempdir().context("Error: Failed to create temp directory")?;
    tracing::info!(
        "Temporary directory created at: {}",
        temp_dir.path().display()
    );
    Ok(temp_dir)
}

pub async fn clear_tmp_dir(temp_dir: TempDir) -> Result<()> {
    let temp_dir_path = temp_dir
        .path()
        .to_str()
        .ok_or(anyhow!("Error: Temp directory parsing fialed"))?
        .to_string();
    temp_dir
        .close()
        .context(format!("Failed to clear {}", temp_dir_path))?;
    tracing::info!("Temporary directory cleared");
    Ok(())
}

pub fn is_ssh_key_exist_localy(params: &Params, key: &str) -> Result<bool> {
    let home_ssh_path = format!("{}/.ssh/known_hosts", params.home_path);
    let known_hosts_content = fs::read_to_string(&home_ssh_path)
        .context(format!("Error: Failed to read {}", home_ssh_path))?;
    Ok(known_hosts_content.contains(&key))
}
