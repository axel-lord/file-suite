use std::{
    fs::File,
    io::{self, BufReader},
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
    str,
};

use clap::{Parser, ValueEnum};
use derive_more::{Display, IsVariant};
use serde_json::Value;
use tap::Pipe;
use thiserror::Error;

/// Program to read a single value from structured text
///
/// Supports json, yaml and toml
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// File to read from
    #[arg(long, short)]
    input: Vec<PathBuf>,

    /// End outputed findings with null
    #[arg(long, short = '0')]
    null: bool,

    /// Stop after first match
    #[arg(long, short = '1', default_value_ifs([("quiet", "true", Some("true")), ("any", "true", Some("true"))]))]
    first: bool,

    /// Do not print anything to stdout
    ///
    /// Implies first
    #[arg(long, short, default_value_if("any", "true", Some("true")))]
    quiet: bool,

    /// Do not limit matches to strings, numbers and bools
    ///
    /// Implies quiet and first
    #[arg(long, short)]
    any: bool,

    /// Filetype of the given files
    ///
    /// Will be applied to all files and Stdin
    ///
    /// If omitted stdin is parsed as json
    #[arg(long = "type", short = 't')]
    file_type: Option<Filetype>,

    /// Path in files to value
    path: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, IsVariant, Default, ValueEnum)]
enum Filetype {
    /// for json
    #[display(fmt = "json")]
    #[default]
    Json,
    /// for yaml
    #[display(fmt = "yaml")]
    Yaml,
    /// for toml
    #[display(fmt = "toml")]
    Toml,
}

#[derive(Clone, Debug, IsVariant)]
enum InputSrc {
    Stdin,
    Path(PathBuf),
}

#[derive(Debug, Error)]
enum Error {
    #[error(transparent)]
    Yaml(#[from] serde_yaml::Error),
    #[error(transparent)]
    Toml(#[from] toml::de::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error("found no matches")]
    NoMatches,
}

impl TryFrom<&Path> for Filetype {
    type Error = ();

    fn try_from(value: &Path) -> Result<Self, Self::Error> {
        fn try_from(value: &Path) -> Option<Filetype> {
            let mut buf: [u8; 4] = [b'\0'; 4];
            let extension = {
                let extension = value.extension()?.as_bytes();
                for (index, &c) in extension.iter().enumerate() {
                    *buf.get_mut(index)? = c;
                }
                buf.make_ascii_lowercase();
                buf.as_slice()
                    .pipe(str::from_utf8)
                    .ok()?
                    .get(0..extension.len())?
            };
            match extension {
                "json" => Filetype::Json,
                "yam" | "yml" => Filetype::Yaml,
                "toml" => Filetype::Toml,
                _ => return None,
            }
            .pipe(Some)
        }
        try_from(value).ok_or(())
    }
}

fn read_to_value(reader: &str, file_type: Filetype) -> Result<Value, Error> {
    match file_type {
        Filetype::Json => serde_json::from_str::<Value>(reader)?,
        Filetype::Yaml => serde_yaml::from_str::<Value>(reader)?,
        Filetype::Toml => toml::from_str::<Value>(reader)?,
    }
    .pipe(Ok)
}

/// Take a function that takes no arguments and call
/// it immedietly, can be used for control flow such as
/// the try operator or return
fn call<T, F: FnOnce() -> T>(f: F) -> T {
    f()
}

fn main() -> Result<(), Error> {
    let Cli {
        input,
        file_type,
        null: null_terminated,
        path,
        first,
        quiet,
        any,
    } = Cli::parse();

    let mut found = 0;
    atty::isnt(atty::Stream::Stdin)
        .then(|| io::stdin().lock())
        .map(|r| {
            (
                call(|| Ok::<_, Error>((io::read_to_string(r)?, file_type.unwrap_or_default()))),
                InputSrc::Stdin,
            )
        })
        .into_iter()
        .chain(input.into_iter().map(|path| {
            (
                call(|| {
                    Ok::<_, Error>((
                        File::open(&path)?
                            .pipe(BufReader::new)
                            .pipe(io::read_to_string)?,
                        file_type.unwrap_or_else(|| {
                            Filetype::try_from(path.as_path()).unwrap_or_default()
                        }),
                    ))
                }),
                InputSrc::Path(path),
            )
        }))
        .filter_map(|(res, i)| {
            Some((
                call(move || {
                    let (r, f) = res?;
                    read_to_value(&r, f)
                })
                .inspect_err(|err| eprintln!("{err}"))
                .ok()?,
                i,
            ))
        })
        .for_each(|(obj, _src)| {
            if first && found >= 1 {
                return;
            }
            let mut obj = &obj;
            for segment in &path {
                let as_int = segment.parse::<usize>();
                if let Some(n_obj) = match (obj, as_int) {
                    (Value::Array(arr), Ok(as_int)) => arr.get(as_int),
                    (Value::Object(map), _) => map.get(segment),
                    _ => return,
                } {
                    obj = n_obj;
                } else {
                    return;
                }
            }

            let obj = match obj {
                Value::Bool(b) => format!("{b}"),
                Value::Number(n) => format!("{n}"),
                Value::String(s) => s.clone(),
                _ if any => String::new(),
                _ => return,
            };

            found += 1;

            if quiet {
                return;
            }

            if null_terminated {
                print!("{obj}\0")
            } else {
                println!("{obj}")
            }
        });

    if found == 0 {
        eprintln!("no data matched using provided path");
        Err(Error::NoMatches)
    } else {
        Ok(())
    }
}
