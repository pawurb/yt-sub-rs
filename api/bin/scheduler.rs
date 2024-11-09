use eyre::Result;
use std::time::Duration;
use yt_sub_api::config::{middleware::init_logs, schedule::get_schedule};

#[tokio::main]
async fn main() -> Result<()> {
    init_logs("scheduler.log");

    let sched = get_schedule().await?;
    sched.start().await?;
    tokio::time::sleep(Duration::MAX).await;

    Ok(())
}
