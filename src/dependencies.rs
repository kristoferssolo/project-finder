use crate::errors::{ProjectFinderError, Result};
use tracing::info;
use which::which;

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

        let fd_path = which("fd")
            .map(|path| path.to_string_lossy().into_owned())
            .map_err(|_| {
                ProjectFinderError::DependencyNotFound(
                    "fd - install from https://github.com/sharkdp/fd".into(),
                )
            })?;

        info!("Found fd at: {fd_path}");

        Ok(Self::new(fd_path))
    }
}
