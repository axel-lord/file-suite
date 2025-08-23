//! Group input by a pattern.

use ::std::{
    borrow::Cow,
    ffi::{OsStr, OsString},
    io::{BufRead, Write, stderr, stdout},
    os::unix::ffi::{OsStrExt, OsStringExt},
    process::{Command, ExitStatus, Stdio},
    str::Utf8Error,
};

use ::clap::{Parser, ValueHint};
use ::rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};
use ::regex::bytes::Captures;
use ::rustc_hash::FxHashSet;
use ::smallvec::SmallVec;

use crate::lookup_chunk::LookupChunk;

mod lookup_chunk;

/// Group input by the result of a regex pattern.
#[derive(Debug, Parser)]
pub struct Cli {
    /// Regex pattern to match.
    regex: String,

    /// Expect null separated/terminated input, and provide null separated output.
    #[arg(long = "null", short = '0', visible_short_alias = 'z')]
    null: bool,

    /// Separate output by null characters.
    #[arg(
        long,
        default_value_if("null", "true", "true"),
        conflicts_with = "null"
    )]
    print0: bool,

    /// Read arguments as separated by null characters.
    #[arg(
        long,
        default_value_if("null", "true", "true"),
        conflicts_with = "null"
    )]
    read0: bool,

    /// Do not skip empty input values, including any produced by trailing linebreaks
    #[arg(long)]
    strict: bool,

    /// Compile the regex as case-insensitive and group by match as uppercase.
    #[arg(long, short)]
    ignore_case: bool,

    /// Group to add all inputs not matching the pattern to.
    ///
    /// If the same as a pattern matched the remainder will be merged
    /// with said group.
    ///
    /// If used without a value the empty string will be used for the group.
    ///
    /// If not specified the remainder will be filtered out.
    #[arg(long, require_equals = true, default_missing_value = "", num_args = 0..=1)]
    remainder: Option<OsString>,

    /// Command to execute for group.
    ///
    /// Captures of a match of the group may be accessed using `{NAME}` or `{?NAME}` syntax,
    /// where the first case requires the capture and the second has it as optional, resolving to
    /// the empty string if missing.
    ///
    /// When remainder is used `{0}` will resolve to the remainder group name, which may be the
    /// empty string. If the remainder group is shared with another group, `{0}` will resolve to
    /// said capture of one of the matches of that group as would be otherwise.
    #[arg(num_args = 1.., trailing_var_arg = true, value_hint = ValueHint::CommandWithArguments)]
    command: Vec<OsString>,
}

/// Crate error type.
#[derive(Debug, ::thiserror::Error)]
pub enum Error {
    /// Could not compile regex.
    #[error(transparent)]
    Re(#[from] ::regex::Error),
    /// Could not read input.
    #[error("while reading input, {0}")]
    InputIO(#[source] ::std::io::Error),
    /// Could not parse format for arguments.
    #[error("could not parse format for argument `{arg}`{msg}")]
    ParseFmt {
        /// Argument that could not be parsed.
        arg: String,
        /// Parse error.
        msg: String,
    },
    /// A capture lookup used a non utf8 byte sequence.
    #[error("non utf8 capture group `{}` used, {}", ::parse_fmt::display_bytes(&.1), .0)]
    NonUtf8CaptureLookup(#[source] Utf8Error, Vec<u8>),
    /// Parsing of the lookup of a capture group failed.
    #[error("could not parse lookup `{}`{}", ::parse_fmt::display_bytes(&chunk), msg)]
    ParseLookup {
        /// Chunk that was to be parsed.
        chunk: Vec<u8>,
        /// Error message.
        msg: String,
    },
    /// A command ran failed.
    #[error("one or more command evocations failed")]
    CommandFailed,
    /// No capture group with given index exists for first match.
    #[error(
        "no capture group with index {idx} exists for \
        used match of pattern /{pattern}/, is the pattern optional?"
    )]
    MissingGroupIdx {
        /// Index of group for which access failed.
        idx: usize,
        /// Regex pattern used.
        pattern: String,
    },
    /// No capture group with given index exists for first match.
    #[error(
        "no capture group with name {name} exists for \
        used match of pattern /{pattern}/, is the pattern optional?"
    )]
    MissingGroupName {
        /// Name of group for which access failed.
        name: String,
        /// Regex pattern used.
        pattern: String,
    },
    /// No capture group with given index exists for pattern.
    #[error(
        "no capture group with index {idx} exists for pattern /{pattern}/, highest group index is {highest}"
    )]
    UnknownGroupIdx {
        /// Index of group.
        idx: usize,
        /// Regex pattern used.
        pattern: String,
        /// Highest group index.
        highest: usize,
    },
    /// No capture group with given name exists for pattern.
    #[error("no capture group with name {name} exists for pattern /{pattern}/")]
    UnknownGroupName {
        /// Name of group.
        name: String,
        /// Regex pattern used.
        pattern: String,
    },
}

#[derive(Debug)]
struct Group<'s> {
    captures: Option<Captures<'s>>,
    inputs: Vec<&'s OsStr>,
}

impl ::file_suite_common::Run for Cli {
    type Error = Error;

    fn run(self) -> Result<(), Self::Error> {
        let Self {
            regex,
            null: _,
            print0,
            read0,
            strict,
            remainder,
            ignore_case,
            command,
        } = self;
        let [exe, command @ ..] = command.as_slice() else {
            panic!("command should contain at least 1 element according to Parser impl")
        };

        let args = command
            .iter()
            .map(|arg| {
                ::parse_fmt::parse_fmt(arg.as_bytes())
                    .collect::<Result<SmallVec<[_; 3]>, _>>()
                    .map_err(|err| {
                        use ::std::fmt::Write;
                        let arg = arg.display().to_string();
                        let msg = match err.as_slice() {
                            [] => String::new(),
                            errors => {
                                let mut out = String::new();
                                for err in errors {
                                    write!(out, "\n{err}").expect("write to string should succeed");
                                }
                                out
                            }
                        };
                        Error::ParseFmt { arg, msg }
                    })
                    .and_then(|chunks| LookupChunk::from_chunks::<SmallVec<[_; 3]>, _>(chunks))
            })
            .collect::<Result<Vec<_>, _>>()?;

        let pattern = regex;
        let regex = ::regex::bytes::RegexBuilder::new(&pattern)
            .case_insensitive(ignore_case)
            .build()?;

        let highest = regex.captures_len() - 1;
        let groups = regex.capture_names().flatten().collect::<FxHashSet<_>>();

        for chunk in args.iter().flatten().copied() {
            match chunk {
                LookupChunk::CaptureIdx(idx) | LookupChunk::CaptureIdxOpt(idx) => {
                    if idx > highest {
                        return Err(Error::UnknownGroupIdx {
                            idx,
                            pattern,
                            highest,
                        });
                    }
                }
                LookupChunk::CaptureName(name) | LookupChunk::CaptureNameOpt(name) => {
                    if !groups.contains(name) {
                        return Err(Error::UnknownGroupName {
                            name: name.to_owned(),
                            pattern,
                        });
                    }
                }
                _ => {}
            }
        }

        let input = ::std::io::stdin()
            .lock()
            .split(if read0 { b'\0' } else { b'\n' })
            .collect::<Result<Vec<_>, ::std::io::Error>>()
            .map_err(Error::InputIO)?;

        let entries = if let Some(remainder) = &remainder {
            input
                .par_iter()
                .filter_map(|haystack| {
                    if !strict && haystack.is_empty() {
                        return None;
                    }
                    Some(if let Some(caps) = regex.captures(haystack) {
                        (
                            if ignore_case {
                                Cow::Owned(OsString::from_vec(::insensitive_buf::to_upper(
                                    &caps[0],
                                )))
                            } else {
                                Cow::Borrowed(OsStr::from_bytes(
                                    caps.get(0)
                                        .expect("capture group 0 should always exist")
                                        .as_bytes(),
                                ))
                            },
                            Some(caps),
                            OsStr::from_bytes(haystack),
                        )
                    } else {
                        (
                            Cow::Borrowed(OsStr::from_bytes(remainder.as_bytes())),
                            None::<Captures>,
                            OsStr::from_bytes(haystack),
                        )
                    })
                })
                .collect::<Vec<_>>()
        } else {
            input
                .par_iter()
                .filter_map(|haystack| {
                    if !strict && haystack.is_empty() {
                        return None;
                    }
                    regex.captures(haystack).map(|caps| {
                        (
                            if ignore_case {
                                Cow::Owned(OsString::from_vec(::insensitive_buf::to_upper(
                                    &caps[0],
                                )))
                            } else {
                                Cow::Borrowed(OsStr::from_bytes(
                                    caps.get(0)
                                        .expect("capture group 0 should always exist")
                                        .as_bytes(),
                                ))
                            },
                            Some(caps),
                            OsStr::from_bytes(haystack),
                        )
                    })
                })
                .collect::<Vec<_>>()
        };

        let mut map = ::hashbrown::HashMap::<_, Group>::new();
        for (key, captures, haystack) in entries {
            use ::hashbrown::hash_map::Entry::{Occupied, Vacant};
            match map.entry(key) {
                Occupied(mut occupied_entry) => {
                    let entry = occupied_entry.get_mut();
                    if entry.captures.is_none() {
                        entry.captures = captures;
                    }
                    entry.inputs.push(haystack);
                }
                Vacant(vacant_entry) => {
                    ::log::info!("{haystack:?}");
                    vacant_entry.insert(Group {
                        captures,
                        inputs: Vec::from([haystack]),
                    });
                }
            }
        }

        let commands = map
            .into_par_iter()
            .map(|(key, value)| {
                // do arg formatting here
                let mut command = Command::new(&exe);

                let mut buf = Vec::<u8>::new();
                for arg in &args {
                    buf.clear();
                    for chunk in arg {
                        match *chunk {
                            LookupChunk::Text(os_str) => buf.extend_from_slice(os_str.as_bytes()),
                            LookupChunk::CaptureIdx(idx) => {
                                if value.captures.is_none() && idx == 0 {
                                    buf.extend_from_slice(key.as_bytes());
                                } else if let Some(captures) = &value.captures {
                                    let r#match = captures.get(idx).ok_or_else(|| {
                                        Error::MissingGroupIdx {
                                            idx,
                                            pattern: pattern.clone(),
                                        }
                                    })?;
                                    buf.extend_from_slice(r#match.as_bytes());
                                } else {
                                    return Err(Error::MissingGroupIdx {
                                        idx,
                                        pattern: pattern.clone(),
                                    });
                                }
                            }
                            LookupChunk::CaptureName(name) => {
                                let Some(captures) = &value.captures else {
                                    return Err(Error::MissingGroupName {
                                        name: name.to_owned(),
                                        pattern: pattern.clone(),
                                    });
                                };
                                let r#match =
                                    captures.name(name).ok_or_else(|| Error::MissingGroupName {
                                        name: name.to_owned(),
                                        pattern: pattern.clone(),
                                    })?;
                                buf.extend_from_slice(r#match.as_bytes());
                            }
                            LookupChunk::CaptureIdxOpt(idx) => {
                                let bytes = value
                                    .captures
                                    .as_ref()
                                    .and_then(|captures| captures.get(idx))
                                    .map(|m| m.as_bytes())
                                    .unwrap_or(&[]);
                                buf.extend_from_slice(bytes);
                            }
                            LookupChunk::CaptureNameOpt(name) => {
                                let bytes = value
                                    .captures
                                    .as_ref()
                                    .and_then(|captures| captures.name(name))
                                    .map(|m| m.as_bytes())
                                    .unwrap_or(&[]);
                                buf.extend_from_slice(bytes);
                            }
                        }
                    }
                    command.arg(OsStr::from_bytes(&buf));
                }

                Ok((command, value.inputs, key))
            })
            .collect::<Result<Vec<_>, Error>>()?;

        fn spawn(
            sep: u8,
            mut command: Command,
            inputs: Vec<&OsStr>,
            group: &OsStr,
        ) -> Result<ExitStatus, ::std::io::Error> {
            let mut child = command
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?;
            let mut stdin = child.stdin.take().expect("stdin pipe should exist");

            for input in inputs {
                stdin.write_all(input.as_bytes())?;
                stdin.write_all(&[sep])?;
            }

            drop(stdin);

            let result = child.wait_with_output()?;

            if !result.stdout.is_empty() {
                let mut stdout = stdout().lock();
                writeln!(stdout, "group <{group}> stdout:", group = group.display())?;
                stdout.write_all(&result.stdout)?;
            }
            if !result.stderr.is_empty() {
                let mut stderr = stderr().lock();
                writeln!(stderr, "group <{group}> stderr:", group = group.display())?;
                stderr.write_all(&result.stderr)?;
            }

            Ok(result.status)
        }

        let sep = if print0 { b'\0' } else { b'\n' };
        let results = commands
            .into_par_iter()
            .map(|(command, inputs, group)| (spawn(sep, command, inputs, &group), group))
            .collect::<Vec<_>>();

        let mut failure = false;
        for (result, group) in results {
            match result {
                Err(err) => {
                    ::log::error!(
                        "io error for group <{group}>\n{err}",
                        group = group.display()
                    );
                    failure = true;
                }
                Ok(status) if !status.success() => {
                    ::log::error!(
                        "command from group <{group}> did not succeed, {status}",
                        group = group.display()
                    );
                    failure = true;
                }
                _ => {}
            }
        }

        if failure {
            Err(Error::CommandFailed)
        } else {
            Ok(())
        }
    }
}
