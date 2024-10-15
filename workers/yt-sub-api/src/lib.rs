use serde_json::{json, Value};
use worker::*;

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let handle = req.headers().get("x-handle").unwrap().unwrap();
    let youtube_api_key = env
        .var("YOUTUBE_API_KEY")
        .expect("Missing YOUTUBE_API_KEY env var");
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
