// use crate::prelude::*;
// use clap::Parser;
// use color_eyre::eyre::eyre;
// use tracing::{debug, error, info};

// use crate::prelude::*;

// mod cli;
// mod config;
// mod prelude;
// // mod event;
// // mod logging;
// // mod token;
// // mod ui;
// // mod utils;

// #[tokio::main]
// async fn main() -> color_eyre::Result<()> {
//     let cli_args = CliArgs::parse();
//     let config = crate::config::Config::from_env()?;
//     crate::logging::init_logging(&cli_args, &config)?;
//     debug!("error handling and logging initialized");

//     info!("authenticating with GitHub...");
//     info!("initializing terminal");
//     let terminal = ratatui::init();

//     info!("starting application main loop");
//     let result = App::new().run(terminal).await;
//     info!("application loop ended, restoring terminal");
//     ratatui::restore();

//     match &result {
//         Ok(_) => info!("application exited successfully"),
//         Err(e) => error!("application exited with error: {}", e),
//     }

//     result
// }

use crate::args::Args;
use crate::error::Result;
use clap::Parser;
use ratatui::style::Color;
use std::process;
use std::time::Duration;
use termbg::Theme;

fn main() -> Result<()> {
    let mut args = Args::parse();
    if args.accent_color.is_none() {
        args.accent_color = termbg::theme(Duration::from_millis(10))
            .map(|theme| {
                if theme == Theme::Dark {
                    Color::White
                } else {
                    Color::Black
                }
            })
            .ok();
    }
    match crate::run(args) {
        Ok(_) => process::exit(0),
        Err(e) => {
            eprintln!("{e}");
            process::exit(1)
        }
    }
}
