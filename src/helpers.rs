use std::fs::{self};

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
    let mut lines = fs::read_to_string(&sops_path)?
        .lines()
        .map(|s| s.to_string())
        .collect::<Vec<_>>();

    if !update_sops_key(&sops_path, key_name, key_value, &mut lines)? {
        add_new_sops_key(&sops_path, key_name, key_value, &mut lines)?;
    }

    Ok(())
}

fn add_new_sops_key(
    sops_path: &str,
    key_name: &str,
    key_value: &str,
    lines: &mut Vec<String>,
) -> Result<()> {
    let new_key_line = format!("    - &{key_name} {key_value}");
    let new_ref_line = format!("          - *{key_name}");

    let mut key_view_index: Option<usize> = None;
    let mut keys: Vec<String> = Vec::new();

    let mut ref_view_index: Option<usize> = None;
    let mut refs: Vec<String> = Vec::new();

    for i in 0..lines.len() {
        if lines[i].trim() == "users: &age_keys" {
            key_view_index = Some(i);
        } else if lines[i].trim() == "- age:" {
            ref_view_index = Some(i);
        }

        if let Some(key_index) = key_view_index {
            if key_index == i - 1 && lines[i].trim_start().starts_with("- &") {
                keys.push(lines[i].trim().to_string());
                key_view_index = Some(i);
            }
        }

        if let Some(ref_index) = ref_view_index {
            if ref_index == i - 1 && lines[i].trim_start().starts_with("- *") {
                refs.push(lines[i].trim().to_string());
                ref_view_index = Some(i);
            }
        }
    }

    let mut new_key_added = false;
    if !keys.iter().any(|key| key == new_key_line.trim()) {
        if let Some(key_index) = key_view_index {
            lines.insert(key_index + 1, new_key_line.to_string());
            new_key_added = true;
        }
    }

    let mut new_ref_added = false;
    if new_key_added && !refs.iter().any(|r#ref| r#ref == new_ref_line.trim()) {
        if let Some(ref_index) = ref_view_index {
            lines.insert(ref_index + 2, new_ref_line.to_string());
            new_ref_added = true;
        }
    }

    if new_key_added && new_ref_added {
        tracing::info!("New key added into SOPS for {key_name}");
        fs::write(sops_path, lines.join("\n"))?;
    } else {
        tracing::warn!("Key was already into SOPS for {key_name}");
    }

    Ok(())
}

fn update_sops_key(
    sops_path: &str,
    key_name: &str,
    key_value: &str,
    lines: &mut Vec<String>,
) -> Result<bool> {
    let key_line_prefix = format!("- &{key_name}");
    let new_key_line = format!("    - &{key_name} {key_value}");

    let existing_key_line = lines
        .iter()
        .enumerate()
        .find(|(_, line)| line.trim_start().starts_with(&key_line_prefix));

    if let Some((line_index, line)) = existing_key_line {
        if line.trim() != new_key_line.trim() {
            lines[line_index] = new_key_line.clone();

            tracing::info!("Update key added into SOPS for {key_name}");
            fs::write(sops_path, lines.join("\n"))?;
            return Ok(true);
        }
    }
    Ok(false)
}
