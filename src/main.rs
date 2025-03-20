mod commands;
mod config;
mod dependencies;
mod errors;
mod finder;

use clap::Parser;
use config::Config;
use dependencies::Dependencies;
use finder::ProjectFinder;
use std::process::exit;
use tracing::{Level, error};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() {
    // Parse CLI arguments
    let config = Config::parse();

    // Setup logging
    let log_level = if config.verbose {
        Level::INFO
    } else {
        Level::ERROR
    };
    let subscriber = FmtSubscriber::builder().with_max_level(log_level).finish();

    if let Err(e) = tracing::subscriber::set_global_default(subscriber) {
        eprintln!("Failed to set up logging: {e}");
        exit(1);
    }

    // Check for required dependencies
    let deps = match Dependencies::check() {
        Ok(deps) => deps,
        Err(e) => {
            error!("{e}");
            eprintln!("Error: {e}");
            eprintln!(
                "This tool requires both 'fd' and 'ripgrep' (rg) to be installed and available in your PATH."
            );
            eprintln!("Please install the missing dependencies and try again.");
            eprintln!("\nInstallation instructions:");
            eprintln!("  fd: https://github.com/sharkdp/fd#installation");
            eprintln!("  ripgrep: https://github.com/BurntSushi/ripgrep#installation");
            exit(1);
        }
    };

    // Create finder and search for projects
    let finder = ProjectFinder::new(config, deps);

    match finder.find_projects().await {
        Ok(projects) => {
            // Output results
            for project in projects {
                println!("{}", project.display());
            }
        }
        Err(e) => {
            error!("Failed to find projects: {e}");
            eprintln!("Error: {e}");
            exit(1);
        }
    }
}
