use crate::common::utils::BASE_DIR;
use anyhow;
use csv::Reader;
use regex::Regex;
use serde::{Deserialize, Deserializer};
use std::{
    fmt::Display,
    fs::{self, File, create_dir_all},
    path::{Path, PathBuf},
    str::FromStr,
    sync::OnceLock,
};
use tempfile::TempDir;

pub static TEMP_DIR: OnceLock<TempDir> = OnceLock::new();

pub fn init_temp_dir() {
    TEMP_DIR.get_or_init(|| setup_entries().expect("Failed to setup test directory"));
}

#[derive(Debug, Clone, Default)]
pub struct BenchParams {
    pub depth: Option<usize>,
    pub max_results: Option<usize>,
    pub verbose: bool,
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

#[allow(dead_code)]
#[derive(Debug, Clone, Default, Deserialize)]
struct Size(#[serde(deserialize_with = "deserialize_u64_from_empty")] u64);

#[allow(dead_code)]
#[derive(Debug, Clone, Default, Deserialize)]
struct Modified(#[serde(deserialize_with = "deserialize_u64_from_empty")] u64);

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
struct Permissions(#[serde(deserialize_with = "deserialize_u16_from_empty")] u16);

pub fn setup_entries() -> anyhow::Result<TempDir> {
    let temp_dir = TempDir::new()?;
    println!("Temporary directory: {:?}", temp_dir.path());

    let fixtures_dir = PathBuf::from(BASE_DIR).join("benches/fixtures");

    let snapshot_path = last_snaphow_file(&fixtures_dir)?;

    let mut rdr = Reader::from_path(snapshot_path)?;

    rdr.deserialize::<FileEntry>()
        .for_each(|entry| match entry {
            Ok(entry) => {
                if let Err(e) = entry.to_tempfile(temp_dir.path()) {
                    eprintln!("Error processing entry: {}", e);
                }
            }
            Err(e) => eprintln!("Failed to deserialize entry: {}", e),
        });

    Ok(temp_dir)
}

fn last_snaphow_file(dir: &Path) -> anyhow::Result<PathBuf> {
    let re = Regex::new(r"^snapshot-(\d{4})-(\d{2})-(\d{2})_(\d{2})-(\d{2})-(\d{2})\.csv$")?;
    let mut snapshots = fs::read_dir(dir)?
        .filter_map(|entry| {
            entry.ok().and_then(|entry| {
                let file_name = entry.file_name();

                if let Some(caps) = re.captures(&file_name.to_string_lossy()) {
                    let [y, m, d, h, min, s] = (1..=6)
                        .filter_map(|i| caps.get(i)?.as_str().parse().ok())
                        .collect::<Vec<u32>>()
                        .try_into()
                        .ok()?;
                    return Some(((y, m, d, h, min, s), entry.path()));
                }
                None
            })
        })
        .collect::<Vec<_>>();

    snapshots.sort_by_key(|(timestamp, _)| *timestamp);

    snapshots
        .last()
        .map(|(_, path)| path.clone())
        .ok_or_else(|| anyhow::anyhow!("No snapshot files found in directory"))
}

fn deserialize_u64_from_empty<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.trim().is_empty() {
        return Ok(0);
    }
    s.parse().map_err(serde::de::Error::custom)
}

fn deserialize_u16_from_empty<'de, D>(deserializer: D) -> Result<u16, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.trim().is_empty() {
        return Ok(644);
    }
    s.parse().map_err(serde::de::Error::custom)
}

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

    fn from_str(s: &str) -> Result<Self, Self::Err> {
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
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl FileEntry {
    fn to_tempfile(&self, base: &Path) -> anyhow::Result<()> {
        let full_path = base.join(&self.path);
        match self.entry_type {
            EntryType::Dir => create_dir(&full_path),
            EntryType::File => create_file(&full_path),
            EntryType::Symlink => Ok(()),
            EntryType::Other(_) => Ok(()),
        }
    }
}

fn create_file(path: &Path) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        create_dir(parent)?;
    }
    File::create(path)?;
    Ok(())
}

fn create_dir(path: &Path) -> anyhow::Result<()> {
    create_dir_all(path)?;
    Ok(())
}

impl Display for BenchParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "depth: {}, max: {}, verbose: {}",
            self.depth.unwrap_or_default(),
            self.max_results.unwrap_or_default(),
            self.verbose
        )
    }
}
