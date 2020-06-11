use anyhow::Result;
use gumdrop::Options;
use log::info;

use crate::queue::Queue;

#[derive(Debug, Options)]
pub struct Args {
    #[options(free, required, help = "command executed to generate job")]
    command: super::exec::Command,
}

pub fn exec(queue: Queue, args: Args) -> Result<()> {
    let stage = queue.push()?;

    info!("Pushing <dir> to queue ({})", stage.id());
    let success = args.command.exec(stage.path(), vec![
        ("QD_JOB_ID", stage.id().as_string())
    ])?;

    if success {
        info!("Job created: {}", stage.id());
        stage.persist()?
    } else {
        info!("Job dismissed: {}", stage.id());
        stage.dismiss()?;
    }

    return Ok(());
}