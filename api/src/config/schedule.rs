use eyre::Result;
use tokio_cron_scheduler::{Job, JobScheduler};

use crate::tasks::{check_videos::run_check_videos, uptime_ping::ping_uptime};

pub async fn get_schedule() -> Result<JobScheduler> {
    let mut sched = JobScheduler::new().await?;

    sched
        .add(Job::new_async("1/10 * * * * *", |_uuid, _l| {
            Box::pin(async move {
                tracing::info!("Checking for new videos");
                match run_check_videos().await {
                    Ok(_) => {}
                    Err(e) => {
                        tracing::error!("Failed to check videos: {}", &e);
                    }
                }
            })
        })?)
        .await?;

    sched
        .add(Job::new_async("1/10 * * * * *", |_uuid, _l| {
            Box::pin(async move {
                tracing::info!("Uptime ping");
                match ping_uptime().await {
                    Ok(_) => {}
                    Err(e) => {
                        tracing::error!("Failed to uptime ping: {}", &e);
                    }
                }
            })
        })?)
        .await?;

    sched.shutdown_on_ctrl_c();

    sched.set_shutdown_handler(Box::new(|| {
        Box::pin(async move {
            tracing::info!("Shut down done");
        })
    }));

    Ok(sched)
}
