use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

use crate::logic::disk::BlockDevice;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Target SSH port
    #[arg(short = 'p', long = "ssh-port", default_value_t = 10022)]
    pub ssh_port: u32,

    /// Target SSH destination
    #[arg(short = 'd', long = "ssh-dest", default_value_t = String::from("127.0.0.1"))]
    pub ssh_dest: String,

    /// Boot from ISO
    #[arg(long, default_value_t = true)]
    pub iso: bool,
}

pub struct Config {
    pub path: Option<PathBuf>,
    pub block_device: Option<BlockDevice>,
    pub hardware_config: Option<Vec<u8>>,
    pub host: Option<String>,
    pub pk_age: Option<String>,
}

impl Config {
    pub fn new() -> Result<Self> {
        Ok(Self {
            path: None,
            block_device: None,
            hardware_config: None,
            host: None,
            pk_age: None,
        })
    }
}
