use anyhow::Result;
use criterion::{Criterion, criterion_group, criterion_main};
use csv::Reader;
use serde::{Deserialize, Deserializer};
use std::{
    fs::{File, create_dir_all},
    path::{Path, PathBuf},
    process::Command,
    str::FromStr,
    u64,
};
use tempfile::TempDir;

const BASE_DIR: &str = env!("CARGO_MANIFEST_DIR");

#[allow(dead_code)]
#[derive(Debug, Clone, Default, Deserialize)]
struct Size(u64);

#[allow(dead_code)]
#[derive(Debug, Clone, Default, Deserialize)]
struct Modified(u64);

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
struct Permissions(u32);

impl Default for Permissions {
    fn default() -> Self {
        Self(644)
    }
}

#[derive(Debug, Clone, PartialEq)]
enum EntryType {
    Dir,
    File,
    Symlink,
    Other(String),
}

impl FromStr for EntryType {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "dir" => Ok(Self::Dir),
            "file" => Ok(Self::File),
            "symlink" => Ok(Self::Symlink),
            other if other.is_empty() => Err("Empty entry type".to_string()),
            other => Ok(Self::Other(other.into())),
        }
    }
}

impl<'de> Deserialize<'de> for EntryType {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(serde::de::Error::custom)
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
struct FileEntry {
    #[serde(rename = "type")]
    entry_type: EntryType,
    directory: PathBuf,
    path: PathBuf,
    #[serde(default)]
    size: Size,
    #[serde(default)]
    modified: Modified,
    #[serde(default)]
    permissions: Permissions,
}

impl FileEntry {
    fn to_tempfile(&self, base: &Path) -> Result<()> {
        let full_path = base.join(&self.path);
        match self.entry_type {
            EntryType::Dir => create_dir(&full_path),
            EntryType::File => create_file(&full_path),
            EntryType::Symlink => Ok(()),
            EntryType::Other(_) => Ok(()),
        }
    }
}

fn create_file(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        create_dir(parent)?;
    }
    File::create(path)?;
    Ok(())
}

fn create_dir(path: &Path) -> Result<()> {
    create_dir_all(path)?;
    Ok(())
}

fn process_directory(path: &Path) {
    let binary_path = PathBuf::from(BASE_DIR).join("target/release/project-finder");
    Command::new(binary_path)
        .arg(path)
        .output()
        .expect("failed to run binary");
}

fn setup_entries() -> Result<TempDir> {
    let temp_dir = TempDir::new()?;
    println!("Temporary directory: {:?}", temp_dir.path());

    let csv_path = PathBuf::from(BASE_DIR)
        .join("benches/fixtures")
        .join("snapshot-2025-03-31_09-20-03.csv");
    let mut rdr = Reader::from_path(csv_path)?;

    rdr.deserialize::<FileEntry>()
        .for_each(|entry| match entry {
            Ok(entry) => {
                if let Err(e) = entry.to_tempfile(temp_dir.path()) {
                    // eprintln!("Error processing entry: {}", e);
                }
            }
            Err(e) => eprintln!("Failed to deserialize entry: {}", e),
        });

    Ok(temp_dir)
}

fn benchmark_processing(c: &mut Criterion) {
    let temp_dir = setup_entries().expect("Failed to setup file entries");

    c.bench_function("process_directory", |b| {
        b.iter(|| process_directory(temp_dir.path()))
    });
}

criterion_group!(benches, benchmark_processing);
criterion_main!(benches);
