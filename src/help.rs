use std::sync::Arc;

use anyhow::Result;
use tempfile::{TempDir, tempdir};
use tokio::{
    io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader},
    sync::Mutex,
};

pub async fn ask_yes_no(prompt: &str) -> Result<bool> {
    let mut stdout = io::stdout();
    stdout
        .write_all(format!("{} [y/N]: ", prompt).as_bytes())
        .await?;
    stdout.flush().await?;

    let mut input = String::new();
    let mut reader = BufReader::new(io::stdin());
    reader.read_line(&mut input).await?;

    Ok(matches!(input.trim().to_lowercase().as_str(), "y" | "yes"))
}

pub fn creat_tmp_dir() -> Result<TempDir> {
    let temp_dir = tempdir()?;
    println!(
        "Temporary directory created at: {}",
        temp_dir.path().display()
    );
    Ok(temp_dir)
}

pub async fn clear_tmp_dir(temp_dir: Arc<Mutex<Option<TempDir>>>) -> Result<()> {
    if let Some(dir) = temp_dir.lock().await.take() {
        dir.close()?;
        println!("Temporary directory cleared");
    } else {
        println!("Temporary directory already cleared");
    }
    Ok(())
}
