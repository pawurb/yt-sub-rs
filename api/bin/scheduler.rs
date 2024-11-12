use eyre::Result;
use std::time::Duration;
use yt_sub_api::config::{middleware, schedule::get_schedule};

#[tokio::main]
async fn main() -> Result<()> {
    match run().await {
        Ok(_) => Ok(()),
        Err(e) => {
            tracing::error!("{:?}", e);
            Err(e)
        }
    }
}

async fn run() -> Result<()> {
    middleware::init_logs("scheduler.log");

    let sched = get_schedule().await?;
    sched.start().await?;
    tokio::time::sleep(Duration::MAX).await;
    Ok(())
}
