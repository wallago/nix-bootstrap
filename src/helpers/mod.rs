use anyhow::{Context, Result};

pub mod input;
pub mod key;

pub fn run_command(cmd: &str) -> Result<()> {
    std::process::Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .status()
        .context("failed to run command")?;
    Ok(())
}
