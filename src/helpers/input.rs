use anyhow::{Context, Result};
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader, Stdout};

pub async fn ask_yes_no(prompt: &str) -> Result<bool> {
    let border = "#".repeat(prompt.len() + 11) + "\n";

    let mut stdout = io::stdout();
    stdout
        .write_all(border.as_bytes())
        .await
        .context("Failed to write in stdout for [y/N]")?;
    let input = enter_input(Some(&mut stdout), &format!("{prompt} [y/N]: ")).await?;
    stdout
        .write_all(border.as_bytes())
        .await
        .context("Failed to write in stdout for [y/N]")?;

    Ok(matches!(input.trim().to_lowercase().as_str(), "y" | "yes"))
}

pub async fn enter_input(stdout: Option<&mut Stdout>, prompt: &str) -> Result<String> {
    let stdout = match stdout {
        Some(stdout) => stdout,
        None => &mut io::stdout(),
    };
    stdout
        .write_all(prompt.as_bytes())
        .await
        .context("Failed to write in stdout for [y/N]")?;
    stdout.flush().await.context("Flushing stdout failed")?;

    let mut input = String::new();
    let mut reader = BufReader::new(io::stdin());
    reader
        .read_line(&mut input)
        .await
        .context("Failed to read stdin for [y/N]")?;
    Ok(input)
}
