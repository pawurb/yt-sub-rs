use axum::{extract::Path, response::IntoResponse};
use eyre::Result;
use reqwest::{
    header::{HeaderMap as ReHeaderMap, CONTENT_TYPE},
    StatusCode,
};
use serde_json::{json, Value};

use crate::config::routes::{invalid_req, json_response};

#[derive(Debug)]
pub struct ChannelData {
    pub channel_id: String,
    pub channel_name: String,
}

pub async fn show(handle: Path<String>) -> impl IntoResponse {
    let response = match show_impl(Some(handle.to_string())).await {
        Ok(Some(response)) => response,
        Ok(None) => {
            return (StatusCode::NOT_FOUND, "Channel not found").into_response();
        }
        Err(e) => return invalid_req(&e.to_string()),
    };

    json_response(response, StatusCode::OK)
}

async fn show_impl(handle: Option<String>) -> Result<Option<Value>> {
    let youtube_api_key =
        std::env::var("YOUTUBE_API_KEY").expect("Missing YOUTUBE_API_KEY env var");

    let handle = match handle {
        Some(handle) => handle,
        None => {
            eyre::bail!("Missing handle")
        }
    };

    let client = reqwest::Client::new();

    let mut headers = ReHeaderMap::new();
    headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());

    let res = client.get(format!("https://www.googleapis.com/youtube/v3/channels?key={}&forHandle={}&part=snippet,id&order=date&maxResults=1", youtube_api_key, handle)).headers(headers).send().await?;
    let status = res.status();

    if status != 200 {
        eyre::bail!("Failed to fetch data {}", status)
    }

    let json: Value = res.json().await?;
    let results = json["pageInfo"]["totalResults"].as_i64().unwrap();

    if results == 0 {
        return Ok(None);
    }

    let channel_id = json["items"][0]["id"].as_str().unwrap();
    let channel_name = json["items"][0]["snippet"]["title"].as_str().unwrap();

    Ok(Some(json!({
        "channel_id": channel_id,
        "channel_name": channel_name
    })))
}
