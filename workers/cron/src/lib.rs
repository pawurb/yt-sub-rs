use eyre::Result as EyreResult;
use kv::KvStore;
use reqwest::Client;
use serde_json::json;
use worker::*;
use yt_sub_core::UserSettings;

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    wasm_rs_dbg::dbg!("Hello from the worker!");
    wasm_rs_dbg::dbg!(req);
    console_error_panic_hook::set_once();
    let dbg_slack_webhook = env
        .var("DBG_SLACK_WEBHOOK")
        .expect("Missing DBG_SLACK_WEBHOOK env var");
    let dbg_slack_webhook = format!("{dbg_slack_webhook}");

    dbg_slack_notify("Hello from the cron!", &dbg_slack_webhook)
        .await
        .unwrap();

    let kv = env.kv("users")?;
    let settings = kv
        .get("7ff01cf2-9ec5-4874-b885-a50c23f292eb")
        .text()
        .await?;
    let settings = settings.unwrap();

    let settings: UserSettings = serde_json::from_str(&settings).unwrap();
    Response::ok("Hello World!")
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
