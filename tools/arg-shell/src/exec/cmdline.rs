//! Execute single command line calls.

use ::std::{
    ffi::OsStr,
    os::unix::ffi::OsStrExt,
    process::{Child, ChildStdin, ChildStdout, Stdio},
};

use crate::exec::{Arg, Exec};

/// [Exec] implementor for cmdline.
#[derive(Debug)]
pub struct Cmdline {
    stdin: ChildStdin,
    stdout: ChildStdout,
    child: Child,
}

impl Exec for Cmdline {}

impl Cmdline {
    /// Create from an ast node.
    pub fn from_ast(node: &mut crate::ast::Cmdline<'_>) -> Result<Self, ::std::io::Error> {
        let mut nodes = node.0.iter_mut();

        let mut command = ::std::process::Command::new(OsStr::from_bytes(
            &crate::exec::arg::Arg::from_ast(
                nodes
                    .next()
                    .expect("all cmdlines should have at least 1 argument"),
            )?
            .get_arg::<Vec<u8>>()?,
        ));

        let mut buf = Vec::<u8>::new();
        for arg in nodes {
            buf.clear();
            crate::exec::arg::Arg::from_ast(arg)?.write_arg(&mut buf)?;
            command.arg(OsStr::from_bytes(&buf));
        }

        let mut child = command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;
        let stdin = child.stdin.take().expect("stdin should exist for child");
        let stdout = child.stdout.take().expect("stdout should exist for child");

        Ok(Self {
            stdin,
            stdout,
            child,
        })
    }
}
