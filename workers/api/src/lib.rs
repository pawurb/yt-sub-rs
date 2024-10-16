mod registration;
use kv::KvStore;
use registration::register_user;
use serde_json::{json, Value};
use worker::*;
use yt_sub_core::UserSettings;

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
        .post_async("/register", |req, _ctx| {
            let mut kv = kv.clone();
            async move { register(req, &mut kv).await }
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

    let api_key = match register_user(settings, kv).await {
        Ok(api_key) => api_key,
        Err(e) => {
            return Response::error(e.to_string(), 400);
        }
    };

    let response = json!({
        "api_key": api_key,
    });

    Response::from_json(&response)
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

    if status == 200 {
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
    } else {
        Response::error("Failed to fetch data", status)
    }
}
