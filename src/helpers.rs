use std::{
    fs::{self, OpenOptions},
    io::Write,
};

use anyhow::{Context, Result, anyhow};
use tempfile::{TempDir, tempdir};
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader, Stdout};

pub async fn ask_yes_no(prompt: &str) -> Result<bool> {
    let border = "#".repeat(prompt.len() + 11) + "\n";

    let mut stdout = io::stdout();
    stdout
        .write_all(border.as_bytes())
        .await
        .context("Error: Failed to write in stdout for [y/N]")?;
    let input = enter_input(Some(&mut stdout), &format!("{prompt} [y/N]: ")).await?;
    stdout
        .write_all(border.as_bytes())
        .await
        .context("Error: Failed to write in stdout for [y/N]")?;

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
    Ok(input)
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

// pub fn is_ssh_fingerprint_is_known(params: &Params, key: &str) -> Result<bool> {
//     let home_ssh_path = format!("{}/.ssh/known_hosts", params.home_path);
//     let known_hosts_content = fs::read_to_string(&home_ssh_path)
//         .context(format!("Error: Failed to read {}", home_ssh_path))?;
//     Ok(known_hosts_content.contains(&key))
// }

// pub fn add_ssh_host_fingerprint(params: &Params) -> Result<()> {
//     let home_ssh_path = format!("{}/.ssh/known_hosts", params.home_path);
//     let host_entry = format!(
//         "[{}]:{} {}",
//         params.target_destination,
//         params.ssh.port,
//         params
//             .ssh
//             .pub_key
//             .clone()
//             .ok_or(anyhow!("error: ssh connection is not established"))?
//     );
//     tracing::info!(
//         "Adding ssh host fingerprint at {} to {}",
//         params.target_destination,
//         home_ssh_path
//     );

//     if !is_ssh_fingerprint_is_known(&params, &host_entry)? {
//         let mut file = OpenOptions::new()
//             .create(true)
//             .write(true)
//             .append(true)
//             .open(&home_ssh_path)
//             .context(format!(
//                 "Error: Failed to open or apppend {}",
//                 home_ssh_path
//             ))?;
//         writeln!(file, "{}", host_entry).context(format!(
//             "Error: Failed to add line {} into {}",
//             params
//                 .ssh
//                 .pub_key
//                 .clone()
//                 .ok_or(anyhow!("error: ssh connection is not established"))?,
//             home_ssh_path
//         ))?;
//     } else {
//         tracing::warn!("Already know the host fingerprint");
//     }
//     Ok(())
// }

pub fn run_command(cmd: &str) -> Result<()> {
    std::process::Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .status()
        .context("failed to run command")?;
    Ok(())
}
