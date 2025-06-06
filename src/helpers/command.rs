use std::process::Command;

use anyhow::{Context, Result, bail};

pub fn run_command(cmd: &str) -> Result<()> {
    let status = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .status()
        .context("failed to run command")?;

    if !status.success() {
        bail!("Host command fail with exit status ({:?})", status)
    }
    Ok(())
}

pub fn run_command_with_stdout(cmd: &str) -> Result<String> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .context("failed to run command")?;

    let stdout = String::from_utf8(output.stdout)?;
    let stderr = String::from_utf8(output.stderr)?;

    if !output.status.success() {
        bail!(
            "Host command fail with exit status ({:?}) and stderr: \n{}",
            output.status,
            stderr
        )
    }
    Ok(stdout)
}
