use crate::errors::{ProjectFinderError, Result};
use std::{
    path::{Path, PathBuf},
    process::Stdio,
};
use tokio::process::Command;
use tracing::{debug, warn};

use crate::dependencies::Dependencies;

/// Run fd command to find files and directories
pub async fn find_files(
    deps: &Dependencies,
    dir: &Path,
    pattern: &str,
    max_depth: usize,
) -> Result<Vec<PathBuf>> {
    let mut cmd = Command::new(&deps.fd_path);

    cmd.arg("--hidden")
        .arg("--no-ignore-vcs")
        .arg("--type")
        .arg("f")
        .arg("--max-depth")
        .arg(max_depth.to_string())
        .arg(pattern)
        .arg(dir)
        .stdout(Stdio::piped());

    debug!("Running: fd {} in {}", pattern, dir.display());

    let output = cmd.output().await.map_err(|e| {
        ProjectFinderError::CommandExecutionFailed(format!("Failed to execute fd: {e}"))
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        warn!("fd command failed: {stderr}");
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8(output.stdout).map_err(ProjectFinderError::Utf8Error)?;

    let paths = stdout.lines().map(PathBuf::from).collect();
    Ok(paths)
}

/// Find Git repositories
pub async fn find_git_repos(
    deps: &Dependencies,
    dir: &Path,
    max_depth: usize,
) -> Result<Vec<PathBuf>> {
    let mut cmd = Command::new(&deps.fd_path);

    cmd.arg("--hidden")
        .arg("--type")
        .arg("d")
        .arg("--max-depth")
        .arg(max_depth.to_string())
        .arg("^.git$")
        .arg(dir)
        .stdout(Stdio::piped());

    debug!("Finding git repos in {}", dir.display());

    let output = cmd.output().await.map_err(|e| {
        ProjectFinderError::CommandExecutionFailed(format!("Failed to find git repositories: {e}"))
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        warn!("fd command failed: {}", stderr);
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8(output.stdout).map_err(ProjectFinderError::Utf8Error)?;

    let paths = stdout
        .lines()
        // Convert .git directories to their parent (the actual repo root)
        .filter_map(|line| {
            let path = PathBuf::from(line);
            path.parent().map(std::path::Path::to_path_buf)
        })
        .collect();

    Ok(paths)
}

/// Run grep on a file to check for a pattern
pub async fn grep_file(deps: &Dependencies, file: &Path, pattern: &str) -> Result<bool> {
    let mut cmd = Command::new(&deps.rg_path);

    cmd.arg("-q") // quiet mode, just return exit code
        .arg(pattern)
        .arg(file);

    let status = cmd.status().await.map_err(|e| {
        ProjectFinderError::CommandExecutionFailed(format!("Failed to execute ripgrep: {e}"))
    })?;

    Ok(status.success())
}
