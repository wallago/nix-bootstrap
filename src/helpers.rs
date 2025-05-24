use std::fs::{self};

use anyhow::{Context, Result, anyhow};
use serde_yaml::Value;
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

pub fn run_command(cmd: &str) -> Result<()> {
    std::process::Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .status()
        .context("failed to run command")?;
    Ok(())
}

pub fn sops_update_age_key(config_path: &str, key_name: &str, key_value: &str) -> Result<()> {
    let sops_path = format!("{}/.sops.yaml", config_path);
    let contents = fs::read_to_string(&sops_path)?;
    let mut sops: Value = serde_yaml::from_str(&contents)?;
    tracing::debug!("SOPS contents:\n{:#?}", sops);
    let sops_copy = sops.clone();
    let key_list = sops
        .get_mut("keys")
        .ok_or(anyhow!("Error: No 'keys' in .sops.yaml: {:?}", sops_copy))?;

    let key_list_debug = key_list.clone();
    let field_list = key_list
        .get_mut("users")
        .ok_or(anyhow!("No 'users' in .sops.yaml: {:?}", key_list_debug))?
        .as_sequence_mut()
        .ok_or(anyhow!(
            "'users' should be a sequence: {:?}",
            key_list_debug
        ))?;

    let mut found = false;
    for entry in field_list.iter_mut() {
        if let Some(anchor) = entry.as_str() {
            tracing::debug!("{anchor}");
            if anchor.contains(&key_name) {
                *entry = Value::String(key_value.to_string());
                found = true;
                tracing::info!("Updated existing {key_name} key");
                break;
            }
        }
    }

    if !found {
        tracing::info!("Adding new {key_name} key with {key_value}");
        field_list.push(Value::String(key_value.to_string()));
    }

    tracing::debug!("New SOPS contents:\n{:#?}", sops);

    // // Convert back to YAML and inject anchors manually
    // let mut out = serde_yaml::to_string(&sops)?;
    // if !found {
    //     let needle = format!("- {}", key_value);
    //     let replacement = format!("- &{} {}", key_name, key_value);
    //     out = out.replacen(&needle, &replacement, 1);
    // }

    // fs::write(&sops_path, out)?;

    Ok(())
}
