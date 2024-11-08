use eyre::Result;

pub async fn run_uptime_ping() -> Result<()> {
    let client = reqwest::Client::new();
    let uptime_url = std::env::var("UPTIME_URL").expect("Missing UPTIME_URL env var");

    let _ = client.get(uptime_url).send().await?;

    Ok(())
}
