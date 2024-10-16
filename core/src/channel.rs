use std::fmt::{self, Display, Formatter};

use chrono::{DateTime, Utc};
use eyre::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{user_settings::API_HOST, video::Video};

const RSS_HOST: &str = "https://www.youtube.com";

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct Channel {
    pub handle: String,
    pub description: String,
    pub channel_id: String,
}

impl Display for Channel {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "name: {name}
handle: {handle}
channel_id: {channel_id}
channel_url: {channel_url}
RSS feed: {channel_feed}",
            name = self.description,
            handle = self.handle,
            channel_id = self.channel_id,
            channel_url = self.url(),
            channel_feed = self.rss_url()
        )
    }
}

impl Channel {
    pub fn url(&self) -> String {
        format!("https://www.youtube.com/{}", self.handle)
    }

    pub fn rss_url(&self) -> String {
        format!(
            "{}/feeds/videos.xml?channel_id={}",
            RSS_HOST, self.channel_id
        )
    }

    pub async fn validate_id(channel_id: &str, host: Option<&str>) -> Result<bool> {
        let host = host.unwrap_or(RSS_HOST);
        let client = Client::new();
        let res = client
            .get(format!(
                "{}/feeds/videos.xml?channel_id={}",
                host, channel_id,
            ))
            .send()
            .await?;

        Ok(res.status() == 200)
    }

    pub async fn get_data(handle: &str, host: Option<&str>) -> Result<(String, String)> {
        let host = host.unwrap_or(API_HOST);
        let client = Client::new();

        let res = client
            .get(format!("{}/channel_data/{}", host, handle))
            .send()
            .await?;
        if res.status() == 404 {
            eyre::bail!("Channel with handle '{handle}' not found!")
        }

        if res.status() == 503 {
            eyre::bail!(
                "It looks like YouTube API calls are currently throttled.

You can try again later or find the channel data manually:
https://github.com/pawurb/yt-sub-rs#manually-finding-an-rss-channel_id"
            );
        }

        let res_json: Value = res.json().await?;
        let channel_id = res_json["channel_id"].as_str().unwrap();
        let channel_name = res_json["channel_name"].as_str().unwrap();

        Ok((channel_id.to_string(), channel_name.to_string()))
    }

    pub async fn get_fresh_videos(&self, last_run_at: DateTime<Utc>) -> Result<Vec<Video>> {
        let rss = self.get_rss_data().await?;
        let videos = Video::parse_rss(rss)?;

        let videos: Vec<Video> = videos
            .into_iter()
            .filter(|video| video.published_at > last_run_at)
            .collect();

        Ok(videos)
    }

    async fn get_rss_data(&self) -> Result<String> {
        let client = Client::new();
        let res = client.get(self.rss_url()).send().await?;
        Ok(res.text().await?)
    }
}

#[cfg(test)]
mod tests {
    use mockito::Server;

    use super::*;
    #[tokio::test]
    async fn test_validate_channel() -> Result<()> {
        let mut server = Server::new_async().await;
        let host = server.host_with_port();
        let host = format!("http://{}", host);
        let m1 = server
            .mock(
                "GET",
                "/feeds/videos.xml?channel_id=UC_iD0xppBwwsrM9DegC5cQQ",
            )
            .with_status(200)
            .create_async()
            .await;
        let m2 = server
            .mock("GET", "/feeds/videos.xml?channel_id=UC_invalid")
            .with_status(404)
            .create_async()
            .await;

        let correct_res = Channel::validate_id("UC_iD0xppBwwsrM9DegC5cQQ", Some(&host)).await?;
        assert!(correct_res);

        let incorrect_res = Channel::validate_id("UC_invalid", Some(&host)).await?;
        assert!(!incorrect_res);

        m1.assert_async().await;
        m2.assert_async().await;

        Ok(())
    }

    // TODO add spec for fetching channel data failed
    #[tokio::test]
    async fn test_get_channel_data() -> Result<()> {
        let mut server = Server::new_async().await;
        let host = server.host_with_port();
        let host = format!("http://{}", host);
        let m = server
            .mock("GET", "/channel_data/@Test_handle")
            .with_body(
                r#"{
            "channel_id": "UC_iD0xppBwwsrM9DegC5cQQ",
            "channel_name": "Test Channel"
        }"#,
            )
            .create_async()
            .await;

        let (channel_id, channel_name) = Channel::get_data("@Test_handle", Some(&host)).await?;
        assert_eq!(channel_id, "UC_iD0xppBwwsrM9DegC5cQQ");
        assert_eq!(channel_name, "Test Channel");

        m.assert_async().await;

        Ok(())
    }
}
