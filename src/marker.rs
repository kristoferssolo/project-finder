use std::{convert::Infallible, str::FromStr};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MarkerType {
    PackageJson,
    CargoToml,
    DenoJson,
    BuildFile(String),
    OtherConfig(String),
}

impl FromStr for MarkerType {
    type Err = Infallible;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(match s {
            "package.json" => Self::PackageJson,
            "Cargo.toml" => Self::CargoToml,
            "deno.json" | "deno.jsonc" => Self::DenoJson,
            "Makefile" | "CMakeLists.txt" | "justfile" | "Justfile" => {
                Self::BuildFile(s.to_string())
            }
            _ => Self::OtherConfig(s.to_string()),
        })
    }
}
