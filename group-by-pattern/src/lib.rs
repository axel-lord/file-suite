//! Group input by a pattern.

use ::std::{
    borrow::Cow,
    collections::VecDeque,
    ffi::{OsStr, OsString},
    io::{BufRead, Write, stderr, stdout},
    os::unix::ffi::{OsStrExt, OsStringExt},
    process::{Command, ExitStatus, Stdio},
};

use ::clap::{Parser, ValueHint};
use ::rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};
use ::regex::bytes::Captures;

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
    /// A command ran failed.
    #[error("one or more command evocations failed")]
    CommandFailed,
}

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

        let mut command = VecDeque::from(command);
        let exe = command
            .pop_front()
            .expect("command should contain at least 1 element according to Parser impl");
        let args = &*command.make_contiguous();

        let regex = ::regex::bytes::RegexBuilder::new(&regex)
            .case_insensitive(ignore_case)
            .build()?;

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
                    vacant_entry.insert(Group {
                        captures,
                        inputs: vec![haystack],
                    });
                }
            }
        }

        let commands = map
            .into_par_iter()
            .map(|(key, value)| {
                // do arg formatting here
                _ = key;
                let mut command = Command::new(&exe);
                command.args(args);
                (command, value.inputs, key)
            })
            .collect::<Vec<_>>();

        fn spawn(
            sep: u8,
            mut command: Command,
            inputs: Vec<&OsStr>,
            group: &OsStr,
        ) -> Result<ExitStatus, ::std::io::Error> {
            let mut child = command.stdin(Stdio::piped()).spawn()?;
            let mut stdin = child.stdin.take().expect("stdin pipe should exist");
            let mut inputs = inputs.into_iter();

            if let Some(first) = inputs.next() {
                stdin.write_all(first.as_bytes())?;
            }

            for input in inputs {
                stdin.write_all(&[sep])?;
                stdin.write_all(input.as_bytes())?;
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
                        "command fro group <{group}> did not succeed, {status}",
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
