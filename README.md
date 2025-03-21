# Project Finder

A command-line tool to discover coding projects in specified directories.
It identifies projects based on common marker files (e.g., `package.json`, `Cargo.toml`, `.git` directories).

## Goal

The goal of this project is to quickly and efficiently locate coding projects within a directory structure.
This is particularly useful for developers working in large codebases or managing multiple repositories.

## Features

* **Fast project discovery:** Quickly scans directories to identify potential projects.
* **Multiple project types:** Recognizes projects based on various marker files for different languages and build systems.
* **Configurable search depth:** Limits the search depth to improve performance.
* **Verbose output:** Provides detailed information about the search process.
* **Workspace Awareness:** Detects and handles workspace configurations correctly, such as Javascript and Rust workspaces.
* **Concurrency:** Uses asynchronous tasks to process multiple directories in parallel, improving performance.

## Requirements

To use Project Finder, you need the following dependencies installed on your system:

* **fd:** A simple, fast, and user-friendly alternative to `find`.
  * Installation instructions: [https://github.com/sharkdp/fd#installation](https://github.com/sharkdp/fd#installation)

These tools must be available in your system's PATH.

## Installation

```bash
cargo install project-finder
```

## Usage

```bash
project-finder [OPTIONS] [PATHS]
```

### Options

* **-d, --depth <DEPTH>**: Maximum search depth (default: 5)
* **-n, --max-results <MAX_RESULTS>**: Maximum number of results to return (default: 0, unlimited)
* **-v, --verbose**: Show verbose output
* **PATHS**: Directories to search for projects (default: ".")

### Examples

* Find projects in the current directory with the default depth:

  ```bash
  project-finder
  ```

* Find projects in a specific directory with a maximum depth of 3:

  ```bash
  project-finder --depth 3 /path/to/search
  ```

* Find projects in multiple directories with verbose output:

```bash
project-finder --verbose /path/to/search1 /path/to/search2
```

* Limit the number of results to 10:

```bash
project-finder --max-results 10
```

## Use Cases

* **Quickly locating projects:** Easily find all projects within a large directory structure.
* **Managing multiple repositories:** Discover all repositories in a directory.
* **Automated scripting:** Integrate project discovery into scripts for build automation, testing, or deployment.
* **Workspace management:** Identify workspace roots for managing multiple related projects.

## License

This project is dual-licensed under either:

* MIT License ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))

at your option.
