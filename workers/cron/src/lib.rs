use eyre::Result as EyreResult;
use kv::KvStore;
use reqwest::Client;
use serde_json::json;
use worker::*;
use yt_sub_core::UserSettings;

#[event(scheduled)]
async fn scheduled(evt: ScheduledEvent, env: Env, _ctx: ScheduleContext) {
    console_error_panic_hook::set_once();
    wasm_rs_dbg::dbg!("Hello from the worker dbg!");
    wasm_rs_dbg::dbg!(evt);
    let dbg_slack_webhook = env
        .var("DBG_SLACK_WEBHOOK")
        .expect("Missing DBG_SLACK_WEBHOOK env var");
    let dbg_slack_webhook = format!("{dbg_slack_webhook}");

    dbg_slack_notify("Hello from the cron!", &dbg_slack_webhook)
        .await
        .unwrap();

    let kv = env.kv("users").unwrap();
    let settings = kv
        .get("7ff01cf2-9ec5-4874-b885-a50c23f292eb")
        .text()
        .await
        .unwrap();
    if let Some(settings) = settings {
        let settings: UserSettings = serde_json::from_str(&settings).unwrap();
    };
}

async fn dbg_slack_notify(message: &str, webhook_url: &str) -> EyreResult<()> {
    let client = Client::new();

    let payload = json!({
        "channel": "yt-sub-dbg",
        "icon_emoji": ":goat:",
        "username": "yt-sub-rs",
        "text": message,
        "unfurl_links": false,
    });

    let res = client.post(webhook_url).json(&payload).send().await?;

    if res.status() == 200 {
        return Ok(());
    }

    let err_msg = res.text().await?;
    eyre::bail!("Failed to send message to Slack: {err_msg}")
}
