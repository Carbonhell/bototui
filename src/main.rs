#![allow(clippy::pedantic)]

use std::env;
use aws_config::BehaviorVersion;
use aws_config::meta::region::RegionProviderChain;
use clap::Parser;
use cli::Cli;
use color_eyre::Result;
use log::info;
use tui_logger::{set_log_file, TuiLoggerFile, TuiLoggerLevelOutput};
use crate::app::App;

mod action;
mod app;
mod cli;
mod components;
mod config;
mod errors;
mod logging;
mod mocks;
mod tui;

#[tokio::main]
async fn main() -> Result<()> {
    tui_logger::init_logger(log::LevelFilter::Info)?;
    tui_logger::set_default_level(log::LevelFilter::Info);
    let mut dir = env::temp_dir();
    dir.push("bototui.log");
    let file_options = TuiLoggerFile::new(dir.to_str().unwrap())
        .output_level(Some(TuiLoggerLevelOutput::Abbreviated))
        .output_file(false)
        .output_separator(':');
    set_log_file(file_options);
    info!(target:"App", "Logging to {}", dir.to_str().unwrap());
    let region_provider = RegionProviderChain::default_provider().or_else("eu-central-1");
    if let Some(region) = region_provider.region().await {
        info!(target: "App", "Using AWS region: {}", region);
    }
    let aws_config = aws_config::defaults(BehaviorVersion::latest())
        .region(region_provider)
        .load()
        .await;

    let _args = Cli::parse();
    let mut app = App::new(aws_config)?;
    app.run().await?;
    Ok(())
}
