use std::fmt::{self, Display, Formatter};

use chrono::{DateTime, Utc};
use eyre::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::video::Video;

const RSS_HOST: &str = "https://www.youtube.com";
const YOUTUBE_API_HOST: &str = "https://www.googleapis.com";

#[derive(Debug, Deserialize)]
struct ChannelsResponse {
    #[serde(default)]
    items: Vec<ChannelItem>,
}

#[derive(Debug, Deserialize)]
struct ChannelItem {
    id: String,
    snippet: ChannelSnippet,
}

#[derive(Debug, Deserialize)]
struct ChannelSnippet {
    title: String,
}

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

    /// Resolves a channel handle to its `(channel_id, channel_name)` using the YouTube Data API.
    pub async fn get_data(
        handle: &str,
        youtube_api_key: &str,
        host: Option<&str>,
    ) -> Result<(String, String)> {
        let host = host.unwrap_or(YOUTUBE_API_HOST);
        let client = Client::new();

        let res = client
            .get(format!("{host}/youtube/v3/channels"))
            .query(&[
                ("key", youtube_api_key),
                ("forHandle", handle),
                ("part", "snippet,id"),
                ("maxResults", "1"),
            ])
            .send()
            .await
            // The URL contains the API key, keep it out of error messages.
            .map_err(|e| e.without_url())?;

        let status = res.status();
        if !status.is_success() {
            let err_msg = res.text().await?;
            eyre::bail!("YouTube API request failed with status {status}: {err_msg}");
        }

        let res: ChannelsResponse = res.json().await?;
        let Some(item) = res.items.into_iter().next() else {
            eyre::bail!("Channel with handle '{handle}' not found!");
        };

        Ok((item.id, item.snippet.title))
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
    use mockito::{Matcher, Server};

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

    #[tokio::test]
    async fn test_get_channel_data() -> Result<()> {
        let mut server = Server::new_async().await;
        let host = format!("http://{}", server.host_with_port());
        let m = server
            .mock("GET", "/youtube/v3/channels")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("key".into(), "API_KEY".into()),
                Matcher::UrlEncoded("forHandle".into(), "@Test_handle".into()),
            ]))
            .with_body(
                r#"{
            "pageInfo": { "totalResults": 1, "resultsPerPage": 1 },
            "items": [
                {
                    "kind": "youtube#channel",
                    "id": "UC_iD0xppBwwsrM9DegC5cQQ",
                    "snippet": { "title": "Test Channel", "description": "" }
                }
            ]
        }"#,
            )
            .create_async()
            .await;

        let (channel_id, channel_name) =
            Channel::get_data("@Test_handle", "API_KEY", Some(&host)).await?;
        assert_eq!(channel_id, "UC_iD0xppBwwsrM9DegC5cQQ");
        assert_eq!(channel_name, "Test Channel");

        m.assert_async().await;

        Ok(())
    }

    #[tokio::test]
    async fn test_get_channel_data_not_found() -> Result<()> {
        let mut server = Server::new_async().await;
        let host = format!("http://{}", server.host_with_port());
        let m = server
            .mock("GET", "/youtube/v3/channels")
            .match_query(Matcher::Any)
            .with_body(r#"{ "pageInfo": { "totalResults": 0, "resultsPerPage": 1 } }"#)
            .create_async()
            .await;

        let err = Channel::get_data("@missing", "API_KEY", Some(&host))
            .await
            .expect_err("missing channel should error");
        assert!(err.to_string().contains("'@missing' not found"));

        m.assert_async().await;
        Ok(())
    }

    #[tokio::test]
    async fn test_get_channel_data_api_error() -> Result<()> {
        let mut server = Server::new_async().await;
        let host = format!("http://{}", server.host_with_port());
        let m = server
            .mock("GET", "/youtube/v3/channels")
            .match_query(Matcher::Any)
            .with_status(400)
            .with_body("API key not valid")
            .create_async()
            .await;

        let err = Channel::get_data("@handle", "BAD_KEY", Some(&host))
            .await
            .expect_err("API error should propagate");
        assert!(err.to_string().contains("API key not valid"));

        m.assert_async().await;
        Ok(())
    }
}
