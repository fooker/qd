use std::convert::TryFrom;
use std::time::{Duration, SystemTime};

use anyhow::Result;
use gumdrop::Options;
use log::{info, trace, warn};

use crate::queue::{Job, NewState, Queue};

#[derive(Debug, Options)]
pub struct Args {
    #[options(help = "scan for new jobs at this interval", default = "5s", parse(try_from_str = "humantime::parse_duration"))]
    scan: Duration,

    #[options(help = "retry failed jobs at this interval", default = "5m", parse(try_from_str = "humantime::parse_duration"))]
    retry: Duration,

    #[options(free, required, help = "command used to process jobs")]
    command: Vec<String>,
}

pub fn exec(queue: Queue, args: Args) -> Result<()> {
    let command = super::exec::Command::try_from(args.command)?;

    let mut next_scan = SystemTime::now() + args.scan;

    loop {
        let now = SystemTime::now();
        trace!("Looping: now={:?}, next_scan={:?}", now, next_scan);

        if now < next_scan {
            trace!("Requeue jobs for retry");
            for failed in queue.failed()? {
                if *failed.since() + args.retry < now {
                    info!("Retrying job {}", failed.id());
                    failed.retry()?;
                }
            }

            trace!("Scanning for new jobs");
            while let Some(job) = queue.poll()? {
                run(job, &command)?;
            }

            next_scan += args.scan;
        }

        std::thread::sleep(Duration::from_secs(1));
    }
}

fn run(job: Job<NewState>, command: &super::exec::Command) -> Result<()> {
    info!("Executing job {}", job.id());

    let success = command.exec(&job.path(), vec![
        ("QD_JOB_ID", job.id().as_string())
    ])?;

    if success {
        info!("Job completed: {}", job.id());
        job.complete()?;
    } else {
        warn!("Job failed: {}", job.id());
        job.error()?;
    };

    return Ok(());
}