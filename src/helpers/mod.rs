use std::process::Stdio;

use anyhow::{Context, Result};

pub mod input;
pub mod temp;

pub fn run_command_with_stdout(cmd: &str) -> Result<()> {
    let status = std::process::Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .status()
        .context("failed to run command")?;

    if !status.success() {
        anyhow::bail!(
            "Host command fail with exit status ({})",
            status.code().unwrap(),
        )
    }
    Ok(())
}

pub fn run_command(cmd: &str) -> Result<String> {
    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .context("failed to run command")?;

    let stdout = String::from_utf8(output.stdout)?;
    let stderr = String::from_utf8(output.stderr)?;

    if !output.status.success() {
        anyhow::bail!(
            "Host command fail with exit status ({}) and stderr: \n{}",
            output.status.code().unwrap(),
            stderr
        )
    }
    Ok(stdout)
}
