mod check_videos;
mod create_account;
mod delete_account;
mod store;
pub mod user_settings_api;
use crate::check_videos::check_videos;
use create_account::create_account;
use delete_account::delete_account;
use kv::KvStore;
use serde_json::{json, Value};
use user_settings_api::UserSettingsAPI;
use wasm_rs_dbg::dbg as wdbg;
use worker::*;
use yt_sub_core::UserSettings;

#[event(scheduled)]
async fn scheduled(_evt: ScheduledEvent, env: Env, _ctx: ScheduleContext) {
    console_error_panic_hook::set_once();
    let mut kv = env.kv("users").expect("Failed to get users kv store");

    let users = match UserSettings::list_ids(&kv).await {
        Ok(users) => users,
        Err(e) => {
            wdbg!("Failed to list users: {}", &e);
            return;
        }
    };

    for user in users {
        match check_videos(user, &mut kv).await {
            Ok(_) => {}
            Err(e) => {
                wdbg!("Failed to check videos: {}", &e);
            }
        }
    }
}

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let youtube_api_key = env
        .var("YOUTUBE_API_KEY")
        .expect("Missing YOUTUBE_API_KEY env var");
    let youtube_api_key = format!("{youtube_api_key}");
    let kv = env.kv("users")?;

    Router::new()
        .get_async("/channel_data/:handle", |_req, ctx| {
            let handle = ctx.param("handle").cloned();

            let youtube_api_key = youtube_api_key.clone();
            async move { channel_data(handle, youtube_api_key).await }
        })
        .post_async("/account", |req, _ctx| {
            let mut kv = kv.clone();
            async move { register(req, &mut kv).await }
        })
        .delete_async("/account", |req, _ctx| {
            let mut kv = kv.clone();
            async move { unregister(req, &mut kv).await }
        })
        .run(req, env)
        .await
}

pub async fn register(req: Request, kv: &mut KvStore) -> Result<Response> {
    let mut req = req.clone().unwrap();
    let body = req.text().await?;
    let settings: UserSettings = match serde_json::from_str(&body) {
        Ok(settings) => settings,
        Err(_) => {
            return Response::error("Invalid settings JSON", 400);
        }
    };

    let api_key = match create_account(settings, kv).await {
        Ok(api_key) => api_key,
        Err(e) => {
            return Response::error(e.to_string(), 400);
        }
    };

    let response = json!({
        "api_key": api_key,
    });

    Ok(Response::from_json(&response).unwrap().with_status(201))
}

pub async fn unregister(req: Request, kv: &mut KvStore) -> Result<Response> {
    let Ok(Some(api_key)) = req.headers().get("X-API-KEY") else {
        return Response::error("Missing X-API-KEY header", 400);
    };

    match delete_account(api_key, kv).await {
        Ok(_) => {}
        Err(e) => {
            return Response::error(e.to_string(), 400);
        }
    };

    Response::ok("DELETED")
}

pub async fn channel_data(handle: Option<String>, youtube_api_key: String) -> Result<Response> {
    let handle = match handle {
        Some(handle) => handle,
        None => return Response::error("Missing handle", 400),
    };

    let mut rss_req = Request::new(
        &format!("https://www.googleapis.com/youtube/v3/channels?key={}&forHandle={}&part=snippet,id&order=date&maxResults=1", youtube_api_key, handle),
        Method::Get,
    )
    .unwrap();

    let headers = rss_req.headers_mut()?;

    headers.append("content-type", "application/json")?;

    let mut res = Fetch::Request(rss_req).send().await?;
    let status = res.status_code();

    if status != 200 {
        return Response::error("Failed to fetch data", status);
    }

    let json: Value = res.json().await?;
    let results = json["pageInfo"]["totalResults"].as_i64().unwrap();

    if results == 0 {
        return Response::error("No channel found", 404);
    }

    let channel_id = json["items"][0]["id"].as_str().unwrap();
    let channel_name = json["items"][0]["snippet"]["title"].as_str().unwrap();
    let response = json!({
        "channel_id": channel_id,
        "channel_name": channel_name,
    });

    Response::from_json(&response)
}
