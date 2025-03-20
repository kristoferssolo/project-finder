use crate::errors::{ProjectFinderError, Result};
use tracing::info;
use which::which;

#[derive(Debug, Clone)]
pub struct Dependencies {
    pub fd_path: String,
    pub rg_path: String,
}

impl Dependencies {
    pub fn new(fd_path: impl Into<String>, rg_path: impl Into<String>) -> Self {
        Self {
            fd_path: fd_path.into(),
            rg_path: rg_path.into(),
        }
    }

    pub fn check() -> Result<Self> {
        info!("Checking dependencies...");

        let fd_path = which("fd").map_err(|_| {
            ProjectFinderError::DependencyNotFound(
                "fd - install from https://github.com/sharkdp/fd".into(),
            )
        })?;

        let rg_path = which("rg").map_err(|_| {
            ProjectFinderError::DependencyNotFound(
                "ripgrep (rg) - install from https://github.com/BurntSushi/ripgrep".into(),
            )
        })?;
        info!("Found fd at: {}", fd_path.display());
        info!("Found ripgrep at: {}", rg_path.display());

        Ok(Self::new(
            fd_path.to_string_lossy(),
            rg_path.to_string_lossy(),
        ))
    }
}
