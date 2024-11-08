use eyre::Result;
use std::time::Duration;
use yt_sub_api::config::schedule::get_schedule;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let sched = get_schedule().await?;
    sched.start().await?;
    tokio::time::sleep(Duration::MAX).await;

    Ok(())
}
