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

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    fn make_manifest(name: &str, description: &str, templates: &[&str]) -> String {
        let files: String = templates
            .iter()
            .map(|t| format!("[[files]]\ntemplate = \"{t}\"\n"))
            .collect::<Vec<_>>()
            .join("\n");
        format!("name = \"{name}\"\ndescription = \"{description}\"\n\n{files}")
    }

    fn create_generator(root: &Path, dir_name: &str, manifest: &str) {
        let gen_dir = root.join(dir_name);
        fs::create_dir_all(&gen_dir).unwrap();
        fs::write(gen_dir.join("generator.toml"), manifest).unwrap();
    }

    #[test]
    fn test_discover_returns_empty_when_path_missing() {
        let tmp = tree_fs::TreeBuilder::default().drop(true).create().unwrap();
        let path = tmp.root.join("templates/generators");
        let result = discover(&path).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_discover_returns_empty_when_no_manifests() {
        let tmp = tree_fs::TreeBuilder::default().drop(true).create().unwrap();
        let gen_dir = tmp.root.join("templates/generators/no-manifest");
        fs::create_dir_all(&gen_dir).unwrap();
        fs::write(gen_dir.join("some.t"), "content").unwrap();

        let result = discover(&tmp.root.join("templates/generators")).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_discover_single_valid_generator() {
        let tmp = tree_fs::TreeBuilder::default().drop(true).create().unwrap();
        let generators_path = tmp.root.join("templates/generators");
        let manifest = make_manifest("my-gen", "My generator", &["controller.t"]);
        create_generator(&generators_path, "my-gen", &manifest);

        let result = discover(&generators_path).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].manifest.name, "my-gen");
        assert_eq!(result[0].manifest.description, "My generator");
        assert_eq!(result[0].manifest.files.len(), 1);
        assert_eq!(result[0].manifest.files[0].template, "controller.t");
    }

    #[test]
    fn test_discover_multiple_generators() {
        let tmp = tree_fs::TreeBuilder::default().drop(true).create().unwrap();
        let generators_path = tmp.root.join("templates/generators");

        create_generator(
            &generators_path,
            "gen-a",
            &make_manifest("gen-a", "Generator A", &["a.t"]),
        );
        create_generator(
            &generators_path,
            "gen-b",
            &make_manifest("gen-b", "Generator B", &["b.t", "b_test.t"]),
        );

        let mut result = discover(&generators_path).unwrap();
        result.sort_by(|a, b| a.manifest.name.cmp(&b.manifest.name));

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].manifest.name, "gen-a");
        assert_eq!(result[1].manifest.name, "gen-b");
        assert_eq!(result[1].manifest.files.len(), 2);
    }

    #[test]
    fn test_discover_invalid_manifest_returns_error() {
        let tmp = tree_fs::TreeBuilder::default().drop(true).create().unwrap();
        let generators_path = tmp.root.join("templates/generators");
        let gen_dir = generators_path.join("bad-gen");
        fs::create_dir_all(&gen_dir).unwrap();
        fs::write(gen_dir.join("generator.toml"), "this is [[[invalid toml").unwrap();

        let result = discover(&generators_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_discover_skips_non_directory_entries() {
        let tmp = tree_fs::TreeBuilder::default().drop(true).create().unwrap();
        let generators_path = tmp.root.join("templates/generators");
        fs::create_dir_all(&generators_path).unwrap();
        fs::write(generators_path.join("stray-file.toml"), "name = \"x\"").unwrap();

        let result = discover(&generators_path).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_find_existing_generator() {
        let tmp = tree_fs::TreeBuilder::default().drop(true).create().unwrap();
        let generators_path = tmp.root.join("templates/generators");
        let manifest = make_manifest("repo", "Repository generator", &["repo.t"]);
        create_generator(&generators_path, "repo", &manifest);

        let gen = find(&generators_path, "repo").unwrap();
        assert_eq!(gen.manifest.name, "repo");
        assert_eq!(gen.manifest.description, "Repository generator");
    }

    #[test]
    fn test_find_missing_generator_returns_error() {
        let tmp = tree_fs::TreeBuilder::default().drop(true).create().unwrap();
        let generators_path = tmp.root.join("templates/generators");

        let result = find(&generators_path, "nonexistent");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("custom generator `nonexistent` not found"));
    }

    #[test]
    fn test_find_selects_correct_generator_among_multiple() {
        let tmp = tree_fs::TreeBuilder::default().drop(true).create().unwrap();
        let generators_path = tmp.root.join("templates/generators");

        create_generator(
            &generators_path,
            "gen-x",
            &make_manifest("gen-x", "X", &["x.t"]),
        );
        create_generator(
            &generators_path,
            "gen-y",
            &make_manifest("gen-y", "Y", &["y.t"]),
        );

        let gen = find(&generators_path, "gen-y").unwrap();
        assert_eq!(gen.manifest.name, "gen-y");
    }
}
