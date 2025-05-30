use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Hostname (ex: nixos) of the nix config
    #[arg(short = 'd', long = "config-hostname")]
    config_hostname: String,
}

pub struct Config {
    pub path: Option<PathBuf>,
    pub hostname: String,
}

impl Config {
    pub fn new(args: Args) -> Result<Self> {
        Ok(Self {
            path: None,
            hostname: args.config_hostname,
        })
    }
}
