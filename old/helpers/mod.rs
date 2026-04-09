use anyhow::Result;
use dialoguer::{Confirm, theme::ColorfulTheme};

pub mod command;
pub mod disk;
pub mod git;

pub fn ask_confirmation(question: &str) -> Result<bool> {
    Ok(Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(question)
        .interact()?)
}
