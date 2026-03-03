// this is because not using with-db renders some of the structs below unused
// TODO: should be more properly aligned with extracting out the db-related gen
// code and then feature toggling it
#![allow(dead_code)]
pub use rrgen::{GenResult, RRgen};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
mod controller;
pub mod custom;
use colored::Colorize;
use std::fmt::Write;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::OnceLock,
};

#[cfg(feature = "with-db")]
mod infer;
#[cfg(feature = "with-db")]
mod migration;
#[cfg(feature = "with-db")]
mod model;
#[cfg(feature = "with-db")]
mod scaffold;
pub mod template;
pub mod tera_ext;

#[derive(Debug)]
pub struct GenerateResults {
    rrgen: Vec<rrgen::GenResult>,
    local_templates: Vec<PathBuf>,
}

impl GenerateResults {
    #[must_use]
    pub fn rrgen_count(&self) -> usize {
        self.rrgen.len()
    }

    #[must_use]
    pub fn local_templates_count(&self) -> usize {
        self.local_templates.len()
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Message(String),
    #[error("template {} not found", path.display())]
    TemplateNotFound { path: PathBuf },
    #[error(transparent)]
    RRgen(#[from] rrgen::Error),
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    Any(#[from] Box<dyn std::error::Error + Send + Sync>),
}

impl Error {
    pub fn msg(err: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self::Message(err.to_string()) //.bt()
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Serialize, Deserialize, Debug)]
struct FieldType {
    name: String,
    rust: RustType,
    schema: String,
    col_type: String,
    #[serde(default)]
    arity: usize,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum RustType {
    String(String),
    Map(HashMap<String, String>),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Mappings {
    field_types: Vec<FieldType>,
}
impl Mappings {
    fn error_unrecognized_default_field(&self, field: &str) -> Error {
        Self::error_unrecognized(field, &self.all_names())
    }

    fn error_unrecognized(field: &str, allow_fields: &[&String]) -> Error {
        Error::Message(format!(
            "type: `{}` not found. try any of: `{}`",
            field,
            allow_fields
                .iter()
                .map(|&s| s.clone())
                .collect::<Vec<String>>()
                .join(",")
        ))
    }

    /// Resolves the Rust type for a given field with optional parameters.
    ///
    /// # Errors
    ///
    /// if rust field not exists or invalid parameters
    pub fn rust_field_with_params(&self, field: &str, params: &Vec<String>) -> Result<&str> {
        match field {
            "array" | "array^" | "array!" => {
                if let RustType::Map(ref map) = self.rust_field_kind(field)? {
                    if let [single] = params.as_slice() {
                        let keys: Vec<&String> = map.keys().collect();
                        Ok(map
                            .get(single)
                            .ok_or_else(|| Self::error_unrecognized(field, &keys))?)
                    } else {
                        Err(self.error_unrecognized_default_field(field))
                    }
                } else {
                    Err(Error::Message(
                        "array field should configured as array".to_owned(),
                    ))
                }
            }

            _ => self.rust_field(field),
        }
    }

    /// Resolves the Rust type for a given field.
    ///
    /// # Errors
    ///
    /// When the given field not recognized
    pub fn rust_field_kind(&self, field: &str) -> Result<&RustType> {
        self.field_types
            .iter()
            .find(|f| f.name == field)
            .map(|f| &f.rust)
            .ok_or_else(|| self.error_unrecognized_default_field(field))
    }

    /// Resolves the Rust type for a given field.
    ///
    /// # Errors
    ///
    /// When the given field not recognized
    pub fn rust_field(&self, field: &str) -> Result<&str> {
        self.field_types
            .iter()
            .find(|f| f.name == field)
            .map(|f| &f.rust)
            .ok_or_else(|| self.error_unrecognized_default_field(field))
            .and_then(|rust_type| match rust_type {
                RustType::String(s) => Ok(s),
                RustType::Map(_) => Err(Error::Message(format!(
                    "type `{field}` need params to get the rust field type"
                ))),
            })
            .map(std::string::String::as_str)
    }

    /// Retrieves the schema field associated with the given field.
    ///
    /// # Errors
    ///
    /// When the given field not recognized
    pub fn schema_field(&self, field: &str) -> Result<&str> {
        self.field_types
            .iter()
            .find(|f| f.name == field)
            .map(|f| f.schema.as_str())
            .ok_or_else(|| self.error_unrecognized_default_field(field))
    }

    /// Retrieves the column type field associated with the given field.
    ///
    /// # Errors
    ///
    /// When the given field not recognized
    pub fn col_type_field(&self, field: &str) -> Result<&str> {
        self.field_types
            .iter()
            .find(|f| f.name == field)
            .map(|f| f.col_type.as_str())
            .ok_or_else(|| self.error_unrecognized_default_field(field))
    }

    /// Retrieves the column type arity associated with the given field.
    ///
    /// # Errors
    ///
    /// When the given field not recognized
    pub fn col_type_arity(&self, field: &str) -> Result<usize> {
        self.field_types
            .iter()
            .find(|f| f.name == field)
            .map(|f| f.arity)
            .ok_or_else(|| self.error_unrecognized_default_field(field))
    }

    #[must_use]
    pub fn all_names(&self) -> Vec<&String> {
        self.field_types.iter().map(|f| &f.name).collect::<Vec<_>>()
    }
}

static MAPPINGS: OnceLock<Mappings> = OnceLock::new();

/// Get type mapping for generation
///
/// # Panics
///
/// Panics if loading fails
pub fn get_mappings() -> &'static Mappings {
    MAPPINGS.get_or_init(|| {
        let json_data = include_str!("./mappings.json");
        serde_json::from_str(json_data).expect("JSON was not well-formatted")
    })
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum ScaffoldKind {
    Api,
    Html,
    Htmx,
}

#[derive(Debug, Clone)]
pub enum DeploymentKind {
    Docker {
        copy_paths: Vec<PathBuf>,
        is_client_side_rendering: bool,
    },
    Nginx {
        host: String,
        port: i32,
    },
}

#[derive(Debug)]
pub enum Component {
    #[cfg(feature = "with-db")]
    Model {
        /// Name of the thing to generate
        name: String,

        /// Whether to include timestamps (`created_at``updated_at`at columns) in the model
        with_tz: bool,

        /// Model fields, eg. title:string hits:int
        fields: Vec<(String, String)>,
    },
    #[cfg(feature = "with-db")]
    Migration {
        /// Name of the migration file
        name: String,

        /// Whether to include timestamps (`created_at`, `updated_at` columns) in the migration
        with_tz: bool,

        /// Params fields, eg. title:string hits:int
        fields: Vec<(String, String)>,
    },
    #[cfg(feature = "with-db")]
    Scaffold {
        /// Name of the thing to generate
        name: String,

        /// Whether to include timestamps (`created_at``updated_at`at columns) in the scaffold
        with_tz: bool,

        /// Model and params fields, eg. title:string hits:int
        fields: Vec<(String, String)>,

        // k
        kind: ScaffoldKind,
    },
    Controller {
        /// Name of the thing to generate
        name: String,

        /// Action names
        actions: Vec<String>,

        // kind
        kind: ScaffoldKind,
    },
    Task {
        /// Name of the thing to generate
        name: String,
    },
    Scheduler {},
    Worker {
        /// Name of the thing to generate
        name: String,
    },
    Mailer {
        /// Name of the thing to generate
        name: String,
    },
    Data {
        /// Name of the thing to generate
        name: String,
    },
    Deployment {
        kind: DeploymentKind,
    },
    Custom {
        /// Name of the custom generator to invoke
        generator_name: String,

        /// Name of the entity to generate
        name: String,

        /// Fields, eg. title:string hits:int
        fields: Vec<(String, String)>,
    },
}

pub struct AppInfo {
    pub app_name: String,
}

/// Indicates whether a generator is built-in or defined by the project.
#[derive(Debug, PartialEq, Eq)]
pub enum GeneratorSource {
    BuiltIn,
    Custom,
}

/// Metadata about an available generator.
#[derive(Debug)]
pub struct GeneratorInfo {
    pub name: String,
    pub description: String,
    pub source: GeneratorSource,
}

/// Returns all available generators: built-in first, then custom from
/// `templates/generators/` in the given base directory.
///
/// # Errors
///
/// Returns an error if custom generator discovery fails.
pub fn list_generators(base_path: &Path) -> Result<Vec<GeneratorInfo>> {
    let mut generators: Vec<GeneratorInfo> = template::list_top_level_dirs()
        .into_iter()
        .map(|name| GeneratorInfo {
            name: name.to_string(),
            description: String::new(),
            source: GeneratorSource::BuiltIn,
        })
        .collect();

    let generators_path = base_path.join(custom::CUSTOM_GENERATORS_PATH);
    for gen in custom::discover(&generators_path)? {
        generators.push(GeneratorInfo {
            name: gen.manifest.name,
            description: gen.manifest.description,
            source: GeneratorSource::Custom,
        });
    }

    Ok(generators)
}

#[must_use]
pub fn new_generator() -> RRgen {
    RRgen::default().add_template_engine(tera_ext::new())
}

/// Generate a component
///
/// # Errors
///
/// This function will return an error if it fails
pub fn generate(rrgen: &RRgen, component: Component, appinfo: &AppInfo) -> Result<GenerateResults> {
    /*
    (1)
    XXX: remove hooks generic from child generator, materialize it here and pass it
         means each generator accepts a [component, config, context] tuple
         this will allow us to test without an app instance
    (2) proceed to test individual generators
     */
    let get_result = match component {
        #[cfg(feature = "with-db")]
        Component::Model {
            name,
            with_tz,
            fields,
        } => model::generate(rrgen, &name, with_tz, &fields, appinfo)?,
        #[cfg(feature = "with-db")]
        Component::Scaffold {
            name,
            with_tz,
            fields,
            kind,
        } => scaffold::generate(rrgen, &name, with_tz, &fields, &kind, appinfo)?,
        #[cfg(feature = "with-db")]
        Component::Migration {
            name,
            with_tz,
            fields,
        } => migration::generate(rrgen, &name, with_tz, &fields, appinfo)?,
        Component::Controller {
            name,
            actions,
            kind,
        } => controller::generate(rrgen, &name, &actions, &kind, appinfo)?,
        Component::Task { name } => {
            let vars = json!({"name": name, "pkg_name": appinfo.app_name});
            render_template(rrgen, Path::new("task"), &vars)?
        }
        Component::Scheduler {} => {
            let vars = json!({"pkg_name": appinfo.app_name});
            render_template(rrgen, Path::new("scheduler"), &vars)?
        }
        Component::Worker { name } => {
            let vars = json!({"name": name, "pkg_name": appinfo.app_name});
            render_template(rrgen, Path::new("worker"), &vars)?
        }
        Component::Mailer { name } => {
            let vars = json!({ "name": name });
            render_template(rrgen, Path::new("mailer"), &vars)?
        }
        Component::Deployment { kind } => match kind {
            DeploymentKind::Docker {
                copy_paths,
                is_client_side_rendering,
            } => {
                let vars = json!({
                    "pkg_name": appinfo.app_name,
                    "copy_paths": copy_paths,
                    "is_client_side_rendering": is_client_side_rendering,
                });
                render_template(rrgen, Path::new("deployment/docker"), &vars)?
            }
            DeploymentKind::Nginx { host, port } => {
                let host = host.replace("http://", "").replace("https://", "");
                let vars = json!({
                    "pkg_name": appinfo.app_name,
                    "domain": host,
                    "port": port
                });
                render_template(rrgen, Path::new("deployment/nginx"), &vars)?
            }
        },
        Component::Data { name } => {
            let vars = json!({ "name": name });
            render_template(rrgen, Path::new("data"), &vars)?
        }
        Component::Custom {
            generator_name,
            name,
            fields,
        } => {
            let generators_path = Path::new(custom::CUSTOM_GENERATORS_PATH);
            let generator = custom::find(generators_path, &generator_name)?;
            let vars = json!({
                "name": name,
                "pkg_name": appinfo.app_name,
                "fields": fields,
            });
            generate_custom(rrgen, &generator, &vars)?
        }
    };

    Ok(get_result)
}

/// Renders all template files defined in a custom generator.
///
/// # Errors
///
/// Returns an error if a template file listed in the manifest cannot be read
/// or if Tera rendering fails.
pub fn generate_custom(
    rrgen: &RRgen,
    generator: &custom::CustomGenerator,
    vars: &Value,
) -> Result<GenerateResults> {
    let mut gen_result = vec![];
    let mut local_templates = vec![];

    for file in &generator.manifest.files {
        let template_path = generator.path.join(&file.template);
        let content = fs::read_to_string(&template_path).map_err(|_| {
            Error::Message(format!(
                "template file `{}` not found in generator `{}`",
                file.template, generator.manifest.name
            ))
        })?;
        gen_result.push(rrgen.generate(&content, vars)?);
        local_templates.push(template_path);
    }

    Ok(GenerateResults {
        rrgen: gen_result,
        local_templates,
    })
}

fn render_template(rrgen: &RRgen, template: &Path, vars: &Value) -> Result<GenerateResults> {
    let template_files = template::collect_files_from_path(template)?;

    let mut gen_result = vec![];
    let mut local_templates = vec![];
    for template in template_files {
        let custom_template = Path::new(template::DEFAULT_LOCAL_TEMPLATE).join(template.path());

        if custom_template.exists() {
            let content = fs::read_to_string(&custom_template).map_err(|err| {
                tracing::error!(custom_template = %custom_template.display(), "could not read custom template");
                err
            })?;
            gen_result.push(rrgen.generate(&content, vars)?);
            local_templates.push(custom_template);
        } else {
            let content = template.contents_utf8().ok_or(Error::Message(format!(
                "could not get template content: {}",
                template.path().display()
            )))?;
            gen_result.push(rrgen.generate(content, vars)?);
        }
    }

    Ok(GenerateResults {
        rrgen: gen_result,
        local_templates,
    })
}

#[must_use]
pub fn collect_messages(results: &GenerateResults) -> String {
    let mut messages = String::new();

    for res in &results.rrgen {
        if let rrgen::GenResult::Generated {
            message: Some(message),
        } = res
        {
            let _ = writeln!(messages, "* {message}");
        }
    }

    if !results.local_templates.is_empty() {
        let _ = writeln!(messages);
        let _ = writeln!(
            messages,
            "{}",
            "The following templates were sourced from the local templates:".green()
        );

        for f in &results.local_templates {
            let _ = writeln!(messages, "* {}", f.display());
        }
    }
    messages
}

/// Copies template files to a specified destination directory.
///
/// This function copies files from the specified template path to the
/// destination directory. If the specified path is `/` or `.`, it copies all
/// files from the templates directory. If the path does not exist in the
/// templates, it returns an error.
///
/// # Errors
/// when could not copy the given template path
pub fn copy_template(path: &Path, to: &Path) -> Result<Vec<PathBuf>> {
    let copy_template_path = if path == Path::new("/") || path == Path::new(".") {
        None
    } else if !template::exists(path) {
        return Err(Error::TemplateNotFound {
            path: path.to_path_buf(),
        });
    } else {
        Some(path)
    };

    let copy_files = if let Some(path) = copy_template_path {
        template::collect_files_from_path(path)?
    } else {
        template::collect_files()
    };

    let mut copied_files = vec![];
    for f in copy_files {
        let copy_to = to.join(f.path());
        if copy_to.exists() {
            tracing::debug!(
                template_file = %copy_to.display(),
                "skipping copy template file. already exists"
            );
            continue;
        }
        match copy_to.parent() {
            Some(parent) => {
                fs::create_dir_all(parent)?;
            }
            None => {
                return Err(Error::Message(format!(
                    "could not get parent folder of {}",
                    copy_to.display()
                )))
            }
        }

        fs::write(&copy_to, f.contents())?;
        tracing::trace!(
            template = %copy_to.display(),
            "copy template successfully"
        );
        copied_files.push(copy_to);
    }
    Ok(copied_files)
}
