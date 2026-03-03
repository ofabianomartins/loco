use loco_gen::template;
use std::path::Path;

#[test]
fn test_list_top_level_dirs_not_empty() {
    let dirs = template::list_top_level_dirs();
    assert!(!dirs.is_empty());
}

#[test]
fn test_list_top_level_dirs_excludes_db_specific_without_feature() {
    let dirs = template::list_top_level_dirs();
    assert!(!dirs.is_empty());
    #[cfg(not(feature = "with-db"))]
    {
        assert!(!dirs.contains(&"scaffold"));
        assert!(!dirs.contains(&"migration"));
        assert!(!dirs.contains(&"model"));
    }
}

#[test]
fn test_exists_known_template_dir() {
    assert!(template::exists(Path::new("task")));
    assert!(!template::exists(Path::new("nonexistent-dir")));
}

#[test]
fn test_exists_known_template_file() {
    // task/task.t is always present in embedded templates
    assert!(template::exists(Path::new("task/task.t")));
    assert!(!template::exists(Path::new("task/none.rs.t")));
}

#[test]
fn test_collect_not_empty() {
    let paths = template::collect();
    assert!(!paths.is_empty());
}

#[test]
fn test_collect_all_paths_exist_in_templates() {
    for path in template::collect() {
        assert!(
            template::exists(&path),
            "collected path {:?} should exist in embedded templates",
            path
        );
    }
}

#[test]
fn test_collect_files_not_empty() {
    let files = template::collect_files();
    assert!(!files.is_empty());
}

#[test]
fn test_collect_files_paths_exist_in_templates() {
    for file in template::collect_files() {
        assert!(
            template::exists(file.path()),
            "file {:?} should exist in embedded templates",
            file.path()
        );
    }
}

#[test]
fn test_get_ignored_paths_with_db_feature() {
    let ignored = template::get_ignored_paths();
    #[cfg(feature = "with-db")]
    assert!(ignored.is_empty());
    #[cfg(not(feature = "with-db"))]
    {
        assert!(ignored.contains(&Path::new("scaffold")));
        assert!(ignored.contains(&Path::new("migration")));
        assert!(ignored.contains(&Path::new("model")));
    }
}

#[test]
fn test_collect_files_from_task_path() {
    let files = template::collect_files_from_path(Path::new("task"))
        .expect("task template dir should exist");
    assert!(!files.is_empty());
}

#[test]
fn test_collect_files_from_nonexistent_path_returns_error() {
    let result = template::collect_files_from_path(Path::new("nonexistent"));
    assert!(result.is_err());
}
