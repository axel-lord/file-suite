#![doc = include_str!("../README.md")]

use ::std::{
    borrow::Cow,
    ffi::OsStr,
    path::{Path, PathBuf},
    process::ExitCode,
    sync::LazyLock,
};

use ::file_suite_common::ExitCodeError;
use ::rayon::prelude::*;
use ::toml_edit::{DocumentMut, InlineTable, Item, Table, Value};

/// Tool to manipulate cargo dependencies.
#[derive(Debug, ::clap::Parser)]
pub struct Cli {
    /// What manipulation to perform.
    #[command(subcommand)]
    tool: Tool,
}

/// Get default path to workspace.
fn default_workspace() -> &'static OsStr {
    static PATH: LazyLock<PathBuf> =
        LazyLock::new(|| PathBuf::from_iter([::std::path::Component::CurDir]));

    PATH.as_os_str()
}

/// How to manipulate dependencies.
#[derive(Debug, ::clap::Subcommand)]
enum Tool {
    /// Sort dependencies.
    Sort {
        /// Path to workspace directory or workspace toml.
        #[arg(long, short, conflicts_with = "cargo_toml", default_value_os = default_workspace())]
        workspace: PathBuf,

        /// Path to cargo toml file or parent directory.
        cargo_toml: Vec<PathBuf>,
    },
    ///Uninherit workspace dependencies.
    Uninherit {
        /// Path to workspace to uninherit.
        #[arg(default_value_os = default_workspace())]
        workspace: PathBuf,
    },
}

impl ::file_suite_common::Run for Cli {
    type Error = ExitCodeError;

    fn run(self) -> Result<(), Self::Error> {
        let Self { tool } = self;
        match tool {
            Tool::Sort {
                workspace,
                cargo_toml,
            } => {
                cargo_toml
                    .par_iter()
                    .map(|path| sort_cargo_toml(path))
                    .reduce(|| Ok(()), reduce_exit_code)?;
                if cargo_toml.is_empty() {
                    sort_workspace(&workspace)?;
                }
            }
            Tool::Uninherit { workspace: _ } => unimplemented!("uninherit is not yet implemented"),
        }
        Ok(())
    }
}

/// Sections which list dependencies.
const DEPENDENCY_SECTIONS: &[&str] = &["dependencies", "dev-dependencies", "build-dependencies"];

/// Fucntion to reduce exit codes.
///
/// # Errors
/// If a and/or b is of the error variant.
const fn reduce_exit_code(
    a: Result<(), ExitCodeError>,
    b: Result<(), ExitCodeError>,
) -> Result<(), ExitCodeError> {
    match (a, b) {
        (a @ Err(_), _) => a,
        (_, b @ Err(_)) => b,
        _ => Ok(()),
    }
}

/// Get a failure exit code error.
///
/// # Errors
/// Always returns FAILURE exit code.
const fn failure<T>() -> Result<T, ExitCodeError> {
    Err(ExitCodeError(ExitCode::FAILURE))
}

/// Move workspace dependencies to members.
///
/// # Errors
/// If workspace or member toml files cannot be read/parsed.
fn uninherit(workspace_path: &Path) -> Result<(), ExitCodeError> {
    let WsPath {
        directory,
        manifest,
    } = WsPath::from(workspace_path);
    let Some(directory) = directory else {
        ::log::error!("workspace path {workspace_path:?} has not parent directory");
        return failure();
    };

    let mut doc = read_doc(&manifest)?;

    let Some(workspace) = doc.get_mut("workspace") else {
        ::log::warn!("cargo manifest at {workspace_path:?} has no workspace section");
        return failure();
    };

    let members = find_members(directory, workspace);

    let Some(workspace_deps) = workspace
        .get("dependencies")
        .and_then(|item| item.as_table())
    else {
        ::log::warn!("cargo manifest at {workspace_path:?} has no workspace dependencies section");
        return failure();
    };

    members
        .into_par_iter()
        .map(|path| {
            let WsPath {
                directory: _,
                manifest,
            } = WsPath::from(path.as_path());
            let mut doc = read_doc(&manifest)?;

            for section in DEPENDENCY_SECTIONS {
                let Some(Item::Table(dependencies)) = doc.get_mut(section) else {
                    continue;
                };

                for (key, value) in dependencies.iter_mut() {
                    let Some(Item::Value(Value::Boolean(is_workspace))) = value.get("workspace")
                    else {
                        continue;
                    };
                    if !is_workspace.value() {
                        continue;
                    }

                    let Some(mut workspace_dep) = workspace_deps.get(&key).cloned() else {
                        ::log::warn!("could not get workspace dependency {key} for {manifest:?}",);
                        continue;
                    };

                    if let Some(features) = value.get("features") {
                        workspace_dep = Item::Value(Value::InlineTable(match workspace_dep {
                            Item::Table(table) => table.into_inline_table(),
                            Item::Value(Value::InlineTable(table)) => table,
                            Item::Value(version @ Value::String(_)) => {
                                let mut table = InlineTable::new();
                                table.insert("version", version);
                                table
                            }
                            _ => {
                                ::log::warn!(
                                    "could not get workspace dependency {key} for {manifest:?}"
                                );
                                continue;
                            }
                        }));

                        if let Some(Item::Value(Value::Array(_features))) =
                            workspace_dep.get_mut("features")
                        {
                        } else {
                            workspace_dep["features"] = features.clone();
                        }
                    }
                }
            }

            Ok(())
        })
        .reduce(|| Ok(()), reduce_exit_code)?;

    if let Item::Table(table) = workspace {
        table.remove("dependencies");
    }

    Ok(())
}

/// Find workspace members of document.
fn find_members(workspace_directory: &Path, workspace: &Item) -> Vec<PathBuf> {
    if let Some(Item::Value(Value::Array(members))) = workspace.get("members") {
        members
            .iter()
            .filter_map(|member| member.as_str())
            .map(|relative| workspace_directory.join(relative))
            .collect()
    } else {
        Vec::new()
    }
}

/// Sort a toml item if possible.
fn sort_item(item: Option<&mut Item>) {
    if let Some(Item::Table(table)) = item {
        table.sort_values();
    }
}

/// Sort dependency headers of table.
fn sort_deps_in_table(table: &mut Table) {
    for section in DEPENDENCY_SECTIONS {
        sort_item(table.get_mut(section));
    }
    if let Some(Item::Table(table)) = table.get_mut("workspace") {
        for section in DEPENDENCY_SECTIONS {
            sort_item(table.get_mut(section));
        }
    }
}

/// Read a path into a toml document.
///
/// # Errors
/// If the file cannot be read/parsed.
fn read_doc(path: &Path) -> Result<DocumentMut, ExitCodeError> {
    let content = ::std::fs::read_to_string(path).map_err(|err| {
        ::log::error!("could not read {path:?}\n{err}");
        ExitCodeError(ExitCode::FAILURE)
    })?;
    let doc = content.parse::<DocumentMut>().map_err(|err| {
        ::log::error!("could not parse {path:?}\n{err}");
        ExitCodeError(ExitCode::FAILURE)
    })?;
    Ok(doc)
}

/// Write a toml document to a path.
///
/// # Errors
/// If the document cannot be written.
fn write_doc(path: &Path, doc: &DocumentMut) -> Result<(), ExitCodeError> {
    ::std::fs::write(path, doc.to_string()).map_err(|err| {
        ::log::error!("could not write to {path:?}\n{err}");
        ExitCodeError(ExitCode::FAILURE)
    })?;
    Ok(())
}

/// Path to workspace.
struct WsPath<'a> {
    /// Directory of workspace.
    directory: Option<&'a Path>,
    /// Toml file of workspace.
    manifest: Cow<'a, Path>,
}

impl<'a> From<&'a Path> for WsPath<'a> {
    fn from(value: &'a Path) -> Self {
        if value
            .file_name()
            .is_some_and(|name| name == OsStr::new("Cargo.toml"))
        {
            WsPath {
                directory: value.parent(),
                manifest: Cow::Borrowed(value),
            }
        } else {
            WsPath {
                directory: Some(value),
                manifest: Cow::Owned(value.join("Cargo.toml")),
            }
        }
    }
}

/// Sort dependencies of cargo toml file.
///
/// # Errors
/// If the file cannot be read/parsed.
fn sort_cargo_toml(path: &Path) -> Result<(), ExitCodeError> {
    let WsPath {
        directory: _,
        manifest,
    } = WsPath::from(path);

    let mut doc = read_doc(&manifest)?;
    sort_deps_in_table(doc.as_table_mut());
    write_doc(&manifest, &doc)?;

    Ok(())
}

/// Sort dependencies of all workspace memebers.
///
/// # Errors
/// If workspace or member toml files cannot be read/parsed.
fn sort_workspace(path: &Path) -> Result<(), ExitCodeError> {
    let WsPath {
        directory,
        manifest,
    } = WsPath::from(path);

    let mut doc = read_doc(&manifest)?;
    sort_deps_in_table(doc.as_table_mut());

    let status = if let Some(workpace) = doc.get_mut("workspace")
        && let Some(Item::Value(Value::Array(members))) = workpace.get_mut("members")
    {
        members.sort_by(|a, b| a.as_str().cmp(&b.as_str()));

        if let Some(directory) = directory {
            members
                .iter()
                .filter_map(|member| member.as_str().map(|rel| directory.join(rel)))
                .collect::<Vec<_>>()
                .into_par_iter()
                .map(|path| sort_cargo_toml(&path))
                .reduce(|| Ok(()), reduce_exit_code)
        } else {
            Ok(())
        }
    } else {
        Ok(())
    };

    write_doc(&manifest, &doc)?;

    status
}
