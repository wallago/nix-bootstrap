use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Target SSH port
    #[arg(short = 'p', long = "ssh-port", default_value_t = 10022)]
    pub ssh_port: u32,

    /// Target SSH destination
    #[arg(short = 'd', long = "ssh-dest", default_value_t = String::from("127.0.0.1"))]
    pub ssh_dest: String,

    /// Target SSH password
    #[arg(short = 'w', long = "ssh-passwd")]
    pub ssh_passwd: Option<String>,
}

pub struct Config {
    pub path: Option<PathBuf>,
}

impl Config {
    pub fn new() -> Result<Self> {
        Ok(Self { path: None })
    }
}
