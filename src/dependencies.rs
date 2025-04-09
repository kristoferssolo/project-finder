use crate::errors::{ProjectFinderError, Result};
use tracing::info;
use which::which;

const FD_PATH: [&str; 2] = ["fd", "fdfind"];

/// Represents external dependencies required by the application.
#[derive(Debug, Clone)]
pub struct Dependencies {
    pub fd_path: String,
}

impl Dependencies {
    /// Creates a new instance of `Dependencies` from the given `fd` binary path.
    pub fn new(fd_path: impl Into<String>) -> Self {
        Self {
            fd_path: fd_path.into(),
        }
    }

    /// Checks if all required dependencies are available, returning an instance of
    /// `Dependencies` with the paths set appropriately.
    ///
    /// At the moment, this only verifies that the `fd` binary is available.
    ///
    /// # Errors
    ///
    /// Returns a `ProjectFinderError::DependencyNotFound` error if `fd` is not found.
    pub fn check() -> Result<Self> {
        info!("Checking dependencies...");

        let fd_path = FD_PATH
            .iter()
            .find_map(|binary| {
                if let Ok(path) = which(binary) {
                    let fd_path = path.to_string_lossy().into_owned();
                    info!("Found {binary} at: {}", fd_path);
                    return Some(fd_path);
                }
                None
            })
            .ok_or_else(|| {
                ProjectFinderError::DependencyNotFound(
                    "Neither 'fd' nor 'fdfind' was found. Please install fd from https://github.com/sharkdp/fd"
                        .into(),
                )
            })?;

        Ok(Self::new(fd_path))
    }
}
