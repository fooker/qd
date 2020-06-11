use std::path::{Path, PathBuf};

use anyhow::Result;
use gumdrop::Options;

use crate::queue::Queue;

#[derive(Debug, Options)]
pub struct Args {
    #[options(help_flag, help = "print help message")]
    pub help: bool,

    #[options(help = "silence all output")]
    pub quiet: bool,

    #[options(count, help = "increase message verbosity")]
    pub verbosity: usize,

    #[options(help = "store queued jobs in this path", meta = "PATH", default = "/var/spool/qd")]
    pub path: PathBuf,

    #[options(required, command)]
    pub command: Option<Command>,
}

#[derive(Debug, Options)]
pub enum Command {
    #[options(help = "executes jobs from the queue")]
    Daemon(daemon::Args),

    #[options(help = "adds a job to the queue")]
    Push(push::Args),

    #[options(help = "print stats about the queue")]
    Stats(stats::Args),
}

impl Command {
    pub fn exec(self, path: impl AsRef<Path>) -> Result<()> {
        let queue = Queue::at(path)?;

        return match self {
            Self::Daemon(args) => daemon::exec(queue, args),
            Self::Push(args) => push::exec(queue, args),
            Self::Stats(args) => stats::exec(queue, args),
        };
    }
}

mod daemon;
mod push;
mod stats;

pub mod exec {
    use std::ffi::OsStr;
    use std::path::Path;
    use std::process;

    use anyhow::{Error, format_err, Result};
    use std::convert::TryFrom;

    #[derive(Debug)]
    pub struct Command {
        program: String,
        arguments: Vec<String>,
    }

    impl Command {
        pub fn exec(&self, dir: impl AsRef<Path>, env: impl IntoIterator<Item=(impl AsRef<OsStr>, impl AsRef<OsStr>)>) -> Result<bool> {
            // Spawn a subprocess executing the job using the jobs path as working directory.
            let mut child = process::Command::new(&self.program)
                .current_dir(dir)
                .args(&self.arguments)
                .envs(env)
                .stdin(process::Stdio::null())
                .stdout(process::Stdio::inherit())
                .stderr(process::Stdio::inherit())
                .spawn()?;

            // Wait for the job to finish and gather status
            let exit_status = child.wait()?;
            return Ok(exit_status.success());
        }
    }

    impl TryFrom<Vec<String>> for Command {
        type Error = Error;

        fn try_from(command: Vec<String>) -> Result<Self, Self::Error> {
            // Separate program (first value) from arguments (remaining values)
            let (program, arguments) = command.split_first()
                .ok_or_else(|| format_err!("Empty command"))?;

            // If there are no arguments given, try to split the program
            let (program, arguments) = if arguments.is_empty() {
                let command = shlex::split(program).ok_or_else(|| format_err!("Can not parse command"))?;
                let (program, arguments) = command.split_first()
                    .ok_or_else(|| format_err!("Empty command"))?;
                (program.to_string(), arguments.to_vec())
            } else {
                (program.to_string(), arguments.to_vec())
            };

            return Ok(Self { program, arguments });
        }
    }

    impl Default for Command {
        fn default() -> Self {
            // FIXME: This is just to make compiler/gumdrop happy. Currently gumdrop requires custom types to impl Default even if they are required.
            return Self { program: String::new(), arguments: vec![] };
        }
    }
}