use eyre::Result;
use serde_json::Value;
use worker::*;

#[derive(Debug)]
pub struct ChannelData {
    pub channel_id: String,
    pub channel_name: String,
}

pub async fn get_channel_data(
    handle: Option<String>,
    youtube_api_key: String,
) -> Result<Option<ChannelData>> {
    let handle = match handle {
        Some(handle) => handle,
        None => {
            eyre::bail!("Missing handle")
        }
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
        eyre::bail!("Failed to fetch data {}", status)
    }

    let json: Value = res.json().await?;
    let results = json["pageInfo"]["totalResults"].as_i64().unwrap();

    if results == 0 {
        return Ok(None);
    }

    let channel_id = json["items"][0]["id"].as_str().unwrap();
    let channel_name = json["items"][0]["snippet"]["title"].as_str().unwrap();

    Ok(Some(ChannelData {
        channel_id: channel_id.to_string(),
        channel_name: channel_name.to_string(),
    }))
}
