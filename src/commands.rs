use crate::{
    dependencies::Dependencies,
    errors::{ProjectFinderError, Result},
};
use regex::{Regex, escape};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    process::Stdio,
};
use tokio::{
    fs::read_to_string,
    io::{AsyncBufReadExt, BufReader},
    process::Command,
};
use tracing::{debug, warn};

/// Run fd command to find files and directories
pub async fn find_files(
    deps: &Dependencies,
    dir: &Path,
    patterns: &[&str],
    max_depth: usize,
) -> Result<HashMap<String, Vec<PathBuf>>> {
    let combined_patterns = format!(
        "({})",
        patterns
            .iter()
            .map(|pattern| escape(pattern))
            .collect::<Vec<_>>()
            .join("|")
    );

    let mut cmd = Command::new(&deps.fd_path);

    cmd.arg("--hidden")
        .arg("--no-ignore-vcs")
        .arg("--type")
        .arg("f")
        .arg("--max-depth")
        .arg(max_depth.to_string())
        .arg(&combined_patterns)
        .arg(dir)
        .stdout(Stdio::piped());

    debug!("Running: fd with combined pattern in {}", dir.display());

    let mut child = cmd.spawn().map_err(|e| {
        ProjectFinderError::CommandExecutionFailed(format!("Failed to spawn fd: {e}"))
    })?;

    // Take the stdout and wrap it with a buffered reader.
    let stdout = child.stdout.take().ok_or_else(|| {
        ProjectFinderError::CommandExecutionFailed("Failed to capture stdout".into())
    })?;
    let reader = BufReader::new(stdout);
    let mut lines = reader.lines();

    let mut results = patterns
        .iter()
        .map(|pattern| ((*pattern).to_string(), Vec::new()))
        .collect::<HashMap<_, _>>();

    // Process output as lines arrive.
    while let Some(line) = lines.next_line().await.map_err(|e| {
        ProjectFinderError::CommandExecutionFailed(format!("Failed to read stdout: {e}"))
    })? {
        let path = PathBuf::from(line);
        if let Some(file_name) = path.file_name().and_then(|f| f.to_str()) {
            if let Some(entries) = results.get_mut(file_name) {
                entries.push(path);
            }
        }
    }

    // Ensure the process has finished.
    let status = child.wait().await.map_err(|e| {
        ProjectFinderError::CommandExecutionFailed(format!("Failed to wait process: {e}"))
    })?;
    if !status.success() {
        warn!("fd command exited with non-zero status: {status}");
    }

    Ok(results)
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

pub async fn grep_file_in_memory(file: &Path, pattern: &str) -> Result<bool> {
    let contents = read_to_string(file).await.map_err(|e| {
        ProjectFinderError::CommandExecutionFailed(format!(
            "Failed to read file {}: {e}",
            file.display()
        ))
    })?;

    let re = Regex::new(pattern).map_err(|e| {
        ProjectFinderError::CommandExecutionFailed(format!("Invalid regex patter {pattern}: {e}"))
    })?;

    Ok(re.is_match(&contents))
}
