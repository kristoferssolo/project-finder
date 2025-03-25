mod commands;
mod config;
mod dependencies;
mod errors;
mod finder;
mod marker;

use crate::{config::Config, dependencies::Dependencies, finder::ProjectFinder};
use clap::Parser;
use std::{error::Error, process::exit};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("{e}");
        exit(1);
    }
}

async fn run() -> Result<(), Box<dyn Error>> {
    // Parse CLI arguments
    let config = Config::parse();

    // Setup logging
    let log_level = if config.verbose {
        Level::INFO
    } else {
        Level::ERROR
    };
    let subscriber = FmtSubscriber::builder().with_max_level(log_level).finish();

    tracing::subscriber::set_global_default(subscriber)
        .map_err(|e| format!("Failed to set up logging: {e}"))?;

    // Check for required dependencies
    let deps = Dependencies::check().map_err(|e| format!("{e}"))?;

    // Create finder and search for projects
    let finder = ProjectFinder::new(config, deps);

    let projects = finder
        .find_projects()
        .await
        .map_err(|e| format!("Failed to find projects: {e}"))?;

    for project in projects {
        println!("{}", project.display());
    }

    Ok(())
}
