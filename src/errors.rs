use std::{path::PathBuf, string::FromUtf8Error};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProjectFinderError {
    #[error("Dependency not found: {0}. Please install it and try again.")]
    DependencyNotFound(String),

    #[error("Failed to execute command: {0}")]
    CommandExecutionFailed(String),

    #[error("Path not found: {0}")]
    PathNotFound(PathBuf),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid UTF-8: {0}")]
    Utf8Error(#[from] FromUtf8Error),
}

pub type Result<T> = std::result::Result<T, ProjectFinderError>;
