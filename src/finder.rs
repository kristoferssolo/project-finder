use crate::{
    commands::{find_files, find_git_repos, grep_file_in_memory},
    config::Config,
    dependencies::Dependencies,
    errors::{ProjectFinderError, Result},
    marker::MarkerType,
};
use futures::future::join_all;
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::{
    fs::metadata,
    spawn,
    sync::{RwLock, Semaphore},
};
use tracing::{debug, info};

type ProjectSet = Arc<RwLock<HashSet<PathBuf>>>;
type WorkspaceCache = Arc<RwLock<HashMap<PathBuf, bool>>>;
type RootCache = Arc<RwLock<HashMap<(PathBuf, String), PathBuf>>>;

const MARKER_PATTERNS: [&str; 13] = [
    "package.json",
    "pnpm-workspace.yaml",
    "lerna.json",
    "Cargo.toml",
    "go.mod",
    "pyproject.toml",
    "CMakeLists.txt",
    "Makefile",
    "justfile",
    "Justfile",
    "deno.json",
    "deno.jsonc",
    "bunfig.toml",
];

async fn path_exists(path: &Path) -> bool {
    metadata(path).await.is_ok()
}

#[derive(Debug, Clone)]
pub struct ProjectFinder {
    config: Config,
    deps: Dependencies,
    discovered_projects: ProjectSet,
    workspace_cache: WorkspaceCache,
    root_cache: RootCache,
}

impl ProjectFinder {
    pub fn new(config: Config, deps: Dependencies) -> Self {
        Self {
            config,
            deps,
            discovered_projects: Arc::new(RwLock::new(HashSet::new())),
            workspace_cache: Arc::new(RwLock::new(HashMap::new())),
            root_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn find_projects(&self) -> Result<Vec<PathBuf>> {
        let semaphore = Arc::new(Semaphore::new(8)); // Limit to 8 concurrent tasks
        let mut handles = vec![];

        for path in &self.config.paths {
            let path_buf = PathBuf::from(path);
            if !path_buf.is_dir() {
                return Err(ProjectFinderError::PathNotFound(path_buf));
            }

            if self.config.verbose {
                info!("Searching in: {}", path);
            }

            let finder_clone = self.clone();
            let path_clone = path_buf.clone();
            let semaphore_clone = Arc::clone(&semaphore);

            // Spawn a task for each directory with semaphore permit
            let handle = spawn(async move {
                let _permit = semaphore_clone.acquire().await.map_err(|e| {
                    ProjectFinderError::CommandExecutionFailed(format!(
                        "Failed to aquire semaphore: {e}"
                    ))
                })?;
                finder_clone.process_directory(&path_clone).await
            });
            handles.push(handle);
        }

        let handle_results = join_all(handles).await;
        let mut errors = handle_results
            .into_iter()
            .filter_map(|handle_result| match handle_result {
                Ok(task_result) => task_result.err().map(|e| {
                    debug!("Task failed: {e}");
                    e
                }),
                Err(e) => {
                    debug!("Task join error: {e}");
                    Some(ProjectFinderError::CommandExecutionFailed(format!(
                        "Task panicked: {e}",
                    )))
                }
            })
            .collect::<Vec<_>>();

        // Return first error if any occurred
        // Only fail if all tasks failed
        if !errors.is_empty() && errors.len() == self.config.paths.len() {
            return Err(errors.remove(0));
        }

        // Return sorted results
        let mut projects = self
            .discovered_projects
            .read()
            .await
            .iter()
            .cloned()
            .collect::<Vec<PathBuf>>();

        projects.sort();

        // Apply max_results if set
        if self.config.max_results > 0 && projects.len() > self.config.max_results {
            projects.truncate(self.config.max_results);
        }

        Ok(projects)
    }

    async fn process_directory(&self, dir: &Path) -> Result<()> {
        // First find all git repositories (usually the most reliable project indicators)
        let git_repos = find_git_repos(&self.deps, dir, self.config.depth).await?;

        {
            self.discovered_projects.write().await.extend(git_repos);
        }

        let marker_map = find_files(&self.deps, dir, &MARKER_PATTERNS, self.config.depth).await?;

        for (pattern, paths) in marker_map {
            for path in paths {
                if let Some(parent_dir) = path.parent() {
                    self.process_marker(parent_dir, &pattern).await?;
                }
            }
        }

        Ok(())
    }

    async fn process_marker(&self, dir: &Path, marker_name: &str) -> Result<()> {
        // Determine marker type
        let marker_type = marker_name.parse().expect("How did we get here?");

        // Find project root
        let project_root = self.find_project_root(dir, &marker_type).await?;

        // Improved nested project detection
        // Only ignore if it's a subproject of the same type (prevents ignoring
        // valid nested projects of different types)
        let mut should_add = true;
        {
            let projects = self.discovered_projects.read().await;
            for known_project in projects.iter() {
                // Check if this is a direct parent (not just any ancestor)
                let is_direct_parent = project_root
                    .parent()
                    .is_some_and(|parent| parent == known_project);

                // Only exclude if it's a subdirectory and has the same marker type
                // or if it's exactly the same directory
                if project_root == *known_project
                    || project_root.starts_with(known_project) && !is_direct_parent
                {
                    should_add = false;
                    break;
                }
            }
        }

        if should_add {
            self.discovered_projects.write().await.insert(project_root);
        }

        Ok(())
    }

    async fn find_project_root(&self, dir: &Path, marker_type: &MarkerType) -> Result<PathBuf> {
        // Check cache
        let cache_key = (dir.to_path_buf(), format!("{marker_type:?}"));
        {
            let cache = self.root_cache.read().await;
            if let Some(root) = cache.get(&cache_key) {
                return Ok(root.clone());
            }
        }

        let mut result = dir.to_path_buf();

        match marker_type {
            MarkerType::PackageJson | MarkerType::DenoJson => {
                // Check for workspace roots
                let mut current = dir.to_path_buf();
                while let Some(parent) = current.parent() {
                    if parent.as_os_str().is_empty() {
                        break;
                    }

                    if self.is_workspace_root(parent).await? {
                        result = parent.to_path_buf();
                        break;
                    }

                    if parent.join(".git").is_dir() {
                        result = parent.to_path_buf();
                        break;
                    }

                    current = parent.to_path_buf();
                }
            }

            MarkerType::CargoToml => {
                // Check for Cargo workspace
                let mut current = dir.to_path_buf();
                while let Some(parent) = current.parent() {
                    if parent.as_os_str().is_empty() {
                        break;
                    }

                    let cargo_toml = parent.join("Cargo.toml");
                    if path_exists(&cargo_toml).await
                        && grep_file_in_memory(&cargo_toml, r"^\[workspace\]").await?
                    {
                        result = parent.to_path_buf();
                        break;
                    }

                    if parent.join(".git").is_dir() {
                        result = parent.to_path_buf();
                        break;
                    }

                    current = parent.to_path_buf();
                }
            }

            MarkerType::BuildFile(name) => {
                // For build system files, find the highest one that's still in the same git repo
                let mut highest_dir = dir.to_path_buf();
                let mut current = dir.to_path_buf();

                while let Some(parent) = current.parent() {
                    if parent.as_os_str().is_empty() {
                        break;
                    }

                    if parent.join(name).exists() {
                        highest_dir = parent.to_path_buf();
                    }

                    if parent.join(".git").is_dir() {
                        result = parent.to_path_buf();
                        break;
                    }

                    current = parent.to_path_buf();
                }

                if result == dir.to_path_buf() {
                    result = highest_dir;
                }
            }

            MarkerType::OtherConfig(_) => {
                // For other file types, just look for git repos
                let mut current = dir.to_path_buf();
                while let Some(parent) = current.parent() {
                    if parent.as_os_str().is_empty() {
                        break;
                    }

                    if parent.join(".git").is_dir() {
                        result = parent.to_path_buf();
                        break;
                    }

                    current = parent.to_path_buf();
                }
            }
        }

        // Cache the result
        self.root_cache
            .write()
            .await
            .insert(cache_key, result.clone());

        Ok(result)
    }

    async fn is_workspace_root(&self, dir: &Path) -> Result<bool> {
        // Check cache
        {
            let cache = self.workspace_cache.read().await;
            if let Some(&result) = cache.get(dir) {
                return Ok(result);
            }
        }

        // Define workspace patterns to check
        let workspace_patterns = [
            (dir.join("package.json"), r#"("workspaces"|"workspace")"#),
            (dir.join("deno.json"), r#"("workspaces"|"imports")"#),
            (dir.join("deno.jsonc"), r#"("workspaces"|"imports")"#),
            (dir.join("bunfig.toml"), r"workspaces"),
            (dir.join("Cargo.toml"), r"^\[workspace\]"),
            (dir.join("rush.json"), r"."),
            (dir.join("nx.json"), r"."),
            (dir.join("turbo.json"), r"."),
        ];

        // Files that indicate workspaces just by existing
        let workspace_files = [
            dir.join("pnpm-workspace.yaml"),
            dir.join("lerna.json"),
            dir.join("yarn.lock"),      // Common in yarn workspaces
            dir.join(".yarnrc.yml"),    // Yarn 2+ workspaces
            dir.join("workspace.json"), // Generic workspace file
        ];

        // Check for workspace by pattern matching
        for (file, pattern) in &workspace_patterns {
            if path_exists(file).await && grep_file_in_memory(file, pattern).await? {
                self.workspace_cache
                    .write()
                    .await
                    .insert(dir.to_path_buf(), true);
                return Ok(true);
            }
        }

        // Check for workspace by file existence
        for file in &workspace_files {
            if path_exists(file).await {
                self.workspace_cache
                    .write()
                    .await
                    .insert(dir.to_path_buf(), true);
                return Ok(true);
            }
        }

        // No workspace found
        self.workspace_cache
            .write()
            .await
            .insert(dir.to_path_buf(), false);
        Ok(false)
    }
}
