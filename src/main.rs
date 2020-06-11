#![feature(never_type)]
#![feature(iterator_fold_self)]

use anyhow::Result;
use gumdrop::Options;
use gumdrop::ParsingStyle::StopAtFirstFree;

use crate::commands::Args;

mod commands;
mod queue;

fn main() -> Result<()> {
    let args: Args = Args::parse_args_or_exit(StopAtFirstFree);

    stderrlog::new()
        .module(module_path!())
        .quiet(args.quiet)
        .verbosity(args.verbosity)
        .init()?;

    let path = args.path;

    return args.command.expect("no command")
        .exec(path);
}
