use anyhow::Result;
use gumdrop::Options;

use crate::queue::Queue;

#[derive(Debug, Options)]
pub struct Args {
}

pub fn exec(queue: Queue, _: Args) -> Result<()> {
    let stats = queue.stats()?;

    println!("queued: {}", stats.queued);
    println!("failed: {}", stats.failed);

    return Ok(());
}