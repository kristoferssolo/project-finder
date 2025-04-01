use std::{
    path::{Path, PathBuf},
    process::Command,
};

pub const BASE_DIR: &str = env!("CARGO_MANIFEST_DIR");

pub fn run_binary_with_args(
    path: &Path,
    depth: usize,
    max_results: usize,
    verbose: bool,
) -> anyhow::Result<()> {
    let binary_path = PathBuf::from(BASE_DIR).join("target/release/project-finder");

    if !binary_path.exists() {
        return Err(anyhow::anyhow!(
            "Binary not found at {}. Did you run `cargo build --release`?",
            binary_path.display()
        ));
    }

    let mut cmd = Command::new(&binary_path);

    // Add the path to search
    cmd.arg(path);

    // Add depth parameter
    cmd.arg("--depth").arg(depth.to_string());

    // Add max_results parameter if not zero
    if max_results > 0 {
        cmd.arg("--max-results").arg(max_results.to_string());
    }

    // Add verbose flag if true
    if verbose {
        cmd.arg("--verbose");
    }

    let output = cmd.output().map_err(|e| {
        anyhow::anyhow!("Failed to execute binary {}: {}", binary_path.display(), e)
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!(
            "Process failed with status: {}\nStderr: {}",
            output.status,
            stderr
        ));
    }

    Ok(())
}
pub fn create_deep_directory(base: &Path, depth: usize) -> anyhow::Result<()> {
    todo!()
}

pub fn create_wide_directory(base: &Path, width: usize) -> anyhow::Result<()> {
    todo!()
}
