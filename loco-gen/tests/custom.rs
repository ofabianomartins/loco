use loco_gen::custom::{discover, find};
use std::fs;
use std::path::Path;

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
