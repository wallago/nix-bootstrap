use std::{net::TcpStream, path::Path};

use anyhow::{Context, Result, anyhow};
use ssh2::Session;
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

pub fn ssh_handshake(params: &Params) -> Result<Session> {
    let tcp = TcpStream::connect(format!("{}:{}", params.target_destination, params.ssh_port))?;
    let mut sess = Session::new()?;
    sess.set_tcp_stream(tcp);
    sess.handshake()?;
    Ok(sess)
}

pub fn ssh_auth(params: &Params, sess: Session) -> Result<Session> {
    sess.userauth_pubkey_file(
        &params.target_user,
        None,
        Path::new(&params.ssh_key_path),
        None,
    )
    .context("Error: SSH authentification failed")?;

    if sess.authenticated() {
        tracing::info!("SSH connection is established");
        Ok(sess)
    } else {
        Err(anyhow!("SSH connection failed"))
    }
}
