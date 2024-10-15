use kv::KvStore;
use serde_json::{json, Value};
use worker::*;

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
        .get_async("/register", |_req, _ctx| {
            let kv = kv.clone();
            async move { register(kv).await }
        })
        .run(req, env)
        .await
}

pub async fn register(kv: KvStore) -> Result<Response> {
    let visits = kv.get("count").text().await?.unwrap_or("0".to_string());
    let visits = visits.parse::<i32>().unwrap_or(0) + 1;
    let _ = kv.put("count", visits.to_string())?.execute().await;
    Response::ok(format!("Tick: {visits}"))
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
