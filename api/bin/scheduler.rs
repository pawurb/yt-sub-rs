use eyre::Result;
use std::time::Duration;
use yt_sub_api::config::schedule::get_schedule;

#[tokio::main]
async fn main() -> Result<()> {
    let file_appender = tracing_appender::rolling::never("./", "scheduler.log");

    tracing_subscriber::fmt().with_writer(file_appender).init();

    let sched = get_schedule().await?;
    sched.start().await?;
    tokio::time::sleep(Duration::MAX).await;

    Ok(())
}
