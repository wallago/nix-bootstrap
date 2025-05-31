use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {}

pub struct Config {
    pub path: Option<PathBuf>,
}

impl Config {
    pub fn new(args: Args) -> Result<Self> {
        Ok(Self { path: None })
    }
}
