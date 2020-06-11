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
    use std::str::FromStr;

    use anyhow::{Error, format_err, Result};

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

    impl FromStr for Command {
        type Err = Error;

        fn from_str(s: &str) -> Result<Self> {
            // Parse the command string using shell splitting
            let command = shlex::split(s)
                .ok_or_else(|| format_err!("Can not parse command"))?;

            // Separate program (first value) from arguments (remaining values)
            let (program, arguments) = command.split_first()
                .ok_or_else(|| format_err!("Empty command"))?;

            return Ok(Self {
                program: program.to_string(),
                arguments: arguments.to_vec(),
            });
        }
    }

    impl Default for Command {
        fn default() -> Self {
            // FIXME: This is just to make compiler/gumdrop happy. Currently gumdrop requires custom types to impl Default even if they are required.
            return Self { program: String::new(), arguments: vec![] };
        }
    }
}