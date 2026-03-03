use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{Error, Result};

pub const CUSTOM_GENERATORS_PATH: &str = "templates/generators";

#[derive(Serialize, Deserialize, Debug)]
pub struct CustomGeneratorFile {
    pub template: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CustomGeneratorManifest {
    pub name: String,
    pub description: String,
    pub files: Vec<CustomGeneratorFile>,
}

#[derive(Debug)]
pub struct CustomGenerator {
    pub manifest: CustomGeneratorManifest,
    pub path: PathBuf,
}

/// Discovers all custom generators in the given directory.
///
/// Each subdirectory containing a `generator.toml` is treated as a generator.
/// Directories without a manifest are silently skipped.
///
/// # Errors
///
/// Returns an error if reading the directory or parsing a manifest fails.
pub fn discover(generators_path: &Path) -> Result<Vec<CustomGenerator>> {
    if !generators_path.exists() {
        return Ok(vec![]);
    }

    let mut generators = vec![];

    for entry in fs::read_dir(generators_path)? {
        let entry = entry?;
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        let manifest_path = path.join("generator.toml");
        if !manifest_path.exists() {
            continue;
        }

        let content = fs::read_to_string(&manifest_path)?;
        let manifest: CustomGeneratorManifest = toml::from_str(&content).map_err(|e| {
            Error::Message(format!(
                "failed to parse generator manifest at {}: {e}",
                manifest_path.display()
            ))
        })?;

        generators.push(CustomGenerator { manifest, path });
    }

    Ok(generators)
}

/// Finds a specific custom generator by name.
///
/// # Errors
///
/// Returns an error if discovery fails or no generator with the given name exists.
pub fn find(generators_path: &Path, name: &str) -> Result<CustomGenerator> {
    discover(generators_path)?
        .into_iter()
        .find(|g| g.manifest.name == name)
        .ok_or_else(|| Error::Message(format!("custom generator `{name}` not found")))
}
