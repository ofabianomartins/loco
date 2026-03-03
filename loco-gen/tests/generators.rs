use loco_gen::{
    collect_messages, copy_template, custom, generate_custom, list_generators, new_generator,
    template, AppInfo, Component, Error, GeneratorSource,
};
use std::{fs, path::Path};

// ── helpers ────────────────────────────────────────────────────────────────

fn make_custom_generator(
    root: &Path,
    name: &str,
    template_content: &str,
) -> custom::CustomGenerator {
    let gen_dir = root.join(name);
    fs::create_dir_all(&gen_dir).unwrap();
    fs::write(gen_dir.join("file.t"), template_content).unwrap();

    custom::CustomGenerator {
        manifest: custom::CustomGeneratorManifest {
            name: name.to_string(),
            description: "test generator".to_string(),
            files: vec![custom::CustomGeneratorFile {
                template: "file.t".to_string(),
            }],
        },
        path: gen_dir,
    }
}

// ── generate_custom ────────────────────────────────────────────────────────

#[test]
fn test_generate_custom_renders_template() {
    let tmp = tree_fs::TreeBuilder::default().drop(true).create().unwrap();
    // rrgen format: YAML header + "---" + Tera body
    let tmpl = "to: \"src/{{ name | snake_case }}.rs\"\n---\n// generated: {{ name }}\n";
    let generator = make_custom_generator(&tmp.root, "my-gen", tmpl);

    let rrgen = new_generator();
    let vars = serde_json::json!({"name": "MyEntity"});
    let result = generate_custom(&rrgen, &generator, &vars).unwrap();

    assert_eq!(result.rrgen_count(), 1);
    assert_eq!(result.local_templates_count(), 1);
}

#[test]
fn test_generate_custom_local_templates_appear_in_messages() {
    let tmp = tree_fs::TreeBuilder::default().drop(true).create().unwrap();
    let tmpl = "to: \"src/{{ name | snake_case }}.rs\"\n---\n// {{ name }}\n";
    let generator = make_custom_generator(&tmp.root, "msg-gen", tmpl);

    let rrgen = new_generator();
    let vars = serde_json::json!({"name": "Item"});
    let result = generate_custom(&rrgen, &generator, &vars).unwrap();
    let messages = collect_messages(&result);

    assert!(
        messages.contains("The following templates were sourced from the local templates:"),
        "messages should mention local templates: {messages}"
    );
}

#[test]
fn test_generate_custom_missing_template_file_returns_error() {
    let tmp = tree_fs::TreeBuilder::default().drop(true).create().unwrap();
    let gen_dir = tmp.root.join("bad-gen");
    fs::create_dir_all(&gen_dir).unwrap();

    let generator = custom::CustomGenerator {
        manifest: custom::CustomGeneratorManifest {
            name: "bad-gen".to_string(),
            description: String::new(),
            files: vec![custom::CustomGeneratorFile {
                template: "missing.t".to_string(),
            }],
        },
        path: gen_dir,
    };

    let rrgen = new_generator();
    let vars = serde_json::json!({"name": "Test"});
    let result = generate_custom(&rrgen, &generator, &vars);

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("template file `missing.t` not found"));
}

#[test]
fn test_generate_custom_multiple_files() {
    let tmp = tree_fs::TreeBuilder::default().drop(true).create().unwrap();
    let gen_dir = tmp.root.join("multi-gen");
    fs::create_dir_all(&gen_dir).unwrap();
    let tmpl = "to: \"src/{{ name | snake_case }}.rs\"\n---\n// {{ name }}\n";
    fs::write(gen_dir.join("a.t"), tmpl).unwrap();
    fs::write(gen_dir.join("b.t"), tmpl).unwrap();

    let generator = custom::CustomGenerator {
        manifest: custom::CustomGeneratorManifest {
            name: "multi-gen".to_string(),
            description: String::new(),
            files: vec![
                custom::CustomGeneratorFile {
                    template: "a.t".to_string(),
                },
                custom::CustomGeneratorFile {
                    template: "b.t".to_string(),
                },
            ],
        },
        path: gen_dir,
    };

    let rrgen = new_generator();
    let vars = serde_json::json!({"name": "Entity"});
    let result = generate_custom(&rrgen, &generator, &vars).unwrap();

    assert_eq!(result.rrgen_count(), 2);
    assert_eq!(result.local_templates_count(), 2);
}

// ── list_generators ────────────────────────────────────────────────────────

#[test]
fn test_list_generators_returns_builtins() {
    let tmp = tree_fs::TreeBuilder::default().drop(true).create().unwrap();
    let result = list_generators(tmp.root.as_path()).unwrap();
    let builtin: Vec<_> = result
        .iter()
        .filter(|g| g.source == GeneratorSource::BuiltIn)
        .collect();
    assert!(
        !builtin.is_empty(),
        "should have at least one built-in generator"
    );
}

#[test]
fn test_list_generators_includes_custom() {
    let tmp = tree_fs::TreeBuilder::default().drop(true).create().unwrap();
    let gen_dir = tmp.root.join(custom::CUSTOM_GENERATORS_PATH).join("my-gen");
    fs::create_dir_all(&gen_dir).unwrap();
    fs::write(
        gen_dir.join("generator.toml"),
        "name = \"my-gen\"\ndescription = \"Custom gen\"\n[[files]]\ntemplate = \"my.t\"\n",
    )
    .unwrap();

    let result = list_generators(tmp.root.as_path()).unwrap();
    let custom: Vec<_> = result
        .iter()
        .filter(|g| g.source == GeneratorSource::Custom)
        .collect();

    assert_eq!(custom.len(), 1);
    assert_eq!(custom[0].name, "my-gen");
    assert_eq!(custom[0].description, "Custom gen");
}

#[test]
fn test_list_generators_no_custom_when_dir_absent() {
    let tmp = tree_fs::TreeBuilder::default().drop(true).create().unwrap();
    let result = list_generators(tmp.root.as_path()).unwrap();
    let custom: Vec<_> = result
        .iter()
        .filter(|g| g.source == GeneratorSource::Custom)
        .collect();
    assert!(custom.is_empty());
}

#[test]
fn test_list_generators_builtin_names_match_template_dirs() {
    let tmp = tree_fs::TreeBuilder::default().drop(true).create().unwrap();
    let result = list_generators(tmp.root.as_path()).unwrap();
    let builtin_names: Vec<&str> = result
        .iter()
        .filter(|g| g.source == GeneratorSource::BuiltIn)
        .map(|g| g.name.as_str())
        .collect();
    let template_dirs = template::list_top_level_dirs();
    for dir in &template_dirs {
        assert!(
            builtin_names.contains(dir),
            "template dir `{dir}` should appear in list_generators"
        );
    }
}

// ── copy_template ──────────────────────────────────────────────────────────

#[test]
fn test_copy_template_nonexistent_returns_error() {
    let tmp = tree_fs::TreeBuilder::default().drop(true).create().unwrap();
    let path = Path::new("nonexistent-template");
    let result = copy_template(path, tmp.root.as_path());
    assert!(result.is_err());
    if let Err(Error::TemplateNotFound { path: p }) = result {
        assert_eq!(p, path.to_path_buf());
    } else {
        panic!("expected TemplateNotFound error");
    }
}

#[test]
fn test_copy_template_copies_task_templates() {
    let tmp = tree_fs::TreeBuilder::default().drop(true).create().unwrap();
    let path = Path::new("task");

    let result = copy_template(path, tmp.root.as_path());
    assert!(result.is_ok(), "copy_template failed: {:?}", result.err());

    let files = template::collect_files_from_path(path).unwrap();
    assert!(!files.is_empty());
    for file in files {
        let dest = tmp.root.join(file.path());
        assert!(dest.exists(), "expected copied file at {:?}", dest);
        let content = fs::read_to_string(&dest).unwrap();
        assert_eq!(
            file.contents_utf8().unwrap(),
            content,
            "content mismatch for {:?}",
            dest
        );
    }
}

// ── Mappings ───────────────────────────────────────────────────────────────

fn test_mapping() -> loco_gen::Mappings {
    serde_json::from_str(
        r#"{
            "field_types": [
                {
                    "name": "array",
                    "rust": {"string": "Vec<String>", "chat": "Vec<String>", "int": "Vec<i32>"},
                    "schema": "array",
                    "col_type": "array_null",
                    "arity": 1
                },
                {
                    "name": "string^",
                    "rust": "String",
                    "schema": "string_uniq",
                    "col_type": "StringUniq"
                }
            ]
        }"#,
    )
    .unwrap()
}

#[test]
fn can_get_all_names_from_mapping() {
    let mapping = test_mapping();
    assert_eq!(
        mapping.all_names(),
        Vec::from([&"array".to_string(), &"string^".to_string()])
    );
}

#[test]
fn can_get_col_type_arity_from_mapping() {
    let mapping = test_mapping();
    assert_eq!(mapping.col_type_arity("array").unwrap(), 1);
    assert_eq!(mapping.col_type_arity("string^").unwrap(), 0);
    assert!(mapping.col_type_arity("unknown").is_err());
}

#[test]
fn can_get_col_type_field_from_mapping() {
    let mapping = test_mapping();
    assert_eq!(mapping.col_type_field("array").unwrap(), "array_null");
    assert!(mapping.col_type_field("unknown").is_err());
}

#[test]
fn can_get_schema_field_from_mapping() {
    let mapping = test_mapping();
    assert_eq!(mapping.schema_field("string^").unwrap(), "string_uniq");
    assert!(mapping.schema_field("unknown").is_err());
}

#[test]
fn can_get_rust_field_from_mapping() {
    let mapping = test_mapping();
    assert_eq!(mapping.rust_field("string^").unwrap(), "String");
    assert!(mapping.rust_field("array").is_err());
    assert!(mapping.rust_field("unknown").is_err());
}

#[test]
fn can_get_rust_field_kind_from_mapping() {
    let mapping = test_mapping();
    assert!(mapping.rust_field_kind("string^").is_ok());
    assert!(mapping.rust_field_kind("unknown").is_err());
}

#[test]
fn can_get_rust_field_with_params_from_mapping() {
    let mapping = test_mapping();
    assert_eq!(
        mapping
            .rust_field_with_params("string^", &vec!["string".to_string()])
            .unwrap(),
        "String"
    );
    assert_eq!(
        mapping
            .rust_field_with_params("array", &vec!["string".to_string()])
            .unwrap(),
        "Vec<String>"
    );
    assert!(mapping
        .rust_field_with_params("array", &vec!["unknown".to_string()])
        .is_err());
    assert!(mapping.rust_field_with_params("unknown", &vec![]).is_err());
}

// ── collect_messages (tested via generate) ────────────────────────────────

#[test]
fn test_collect_messages_shows_task_message() {
    let tmp = tree_fs::TreeBuilder::default()
        .drop(true)
        .add_empty("src/tasks/mod.rs")
        .add_empty("tests/requests/mod.rs")
        .add_empty("tests/tasks/mod.rs")
        .add(
            "src/app.rs",
            "impl Hooks for App {\n    #[allow(unused_variables)]\n    fn register_tasks(tasks: &mut Tasks) {\n        // tasks-inject (do not remove)\n    }\n",
        )
        .create()
        .unwrap();

    let rrgen = rrgen::RRgen::with_working_dir(&tmp.root);
    let result = loco_gen::generate(
        &rrgen,
        Component::Task {
            name: "cleanup".to_string(),
        },
        &AppInfo {
            app_name: "tester".to_string(),
        },
    )
    .unwrap();

    let messages = collect_messages(&result);
    assert!(
        messages.contains("Cleanup"),
        "expected task name in messages: {messages}"
    );
}
