use std::sync::Arc;

use anyhow::{Context, Result, anyhow};
use tempfile::{TempDir, tempdir};
use tokio::{
    io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader},
    sync::Mutex,
};

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

pub async fn clear_tmp_dir(temp_dir: Arc<Mutex<Option<TempDir>>>) -> Result<()> {
    if let Some(dir) = temp_dir.lock().await.take() {
        let dir_path = dir
            .path()
            .to_str()
            .ok_or(anyhow!("Error: Temp directory parsing fialed"))?
            .to_string();
        dir.close()
            .context(format!("Failed to clear {}", dir_path))?;
        tracing::info!("Temporary directory cleared");
    } else {
        tracing::info!("Temporary directory already cleared");
    }
    Ok(())
}
