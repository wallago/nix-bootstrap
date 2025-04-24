use std::sync::Arc;

use anyhow::Result;
use clap::Parser;
use tokio::{signal, sync::Mutex};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {}

mod help;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let temp_dir = Arc::new(Mutex::new(Some(help::creat_tmp_dir()?)));
    let temp_dir_copy = temp_dir.clone();

    tokio::spawn(async move {
        signal::ctrl_c().await.expect("Failed to listen for Ctrl-C");
        if let Err(e) = help::clear_tmp_dir(temp_dir_copy).await {
            eprintln!("Failed to clear tmp dir: {e}");
        }
        std::process::exit(130);
    });

    if help::ask_yes_no("Do you want to continue.?").await? {
        println!("You chose yes.");
    } else {
        println!("You chose no or cancelled.");
    }

    help::clear_tmp_dir(temp_dir).await?;

    Ok(())
}
