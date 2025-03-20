use clap::Parser;

#[derive(Debug, Parser, Clone)]
#[clap(
    author,
    version,
    about = "Find coding projects in specified directories"
)]
pub struct Config {
    /// Directories to search for projects
    #[clap(default_value = ".")]
    pub paths: Vec<String>,

    /// Maximum search depth
    #[clap(short, long, default_value = "5")]
    pub depth: usize,

    /// Show verbose output
    #[clap(short, long)]
    pub verbose: bool,
}
