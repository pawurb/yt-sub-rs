use chrono::{DateTime, Utc};
use eyre::Result;
use xmltojson::to_json;

use crate::notifier::Notifier;

#[derive(Debug)]
pub struct Video {
    pub channel: String,
    pub title: String,
    pub link: String,
    pub published_at: DateTime<Utc>,
}

impl Video {
    pub fn parse_rss(rss_data: String) -> Result<Vec<Video>> {
        let mut videos = vec![];
        let json = to_json(&rss_data).expect("Failed to convert XML to JSON");
        let channel = json["feed"]["author"]["name"].as_str().unwrap();
        let videos_data = json["feed"]["entry"].as_array().unwrap();

        for video_data in videos_data {
            let title = video_data["title"].as_str().unwrap();
            let published_at = video_data["published"].as_str().unwrap();
            let published_at: DateTime<Utc> =
                published_at.parse().expect("Failed to parse DateTime");
            let link = video_data["link"]["@href"].as_str().unwrap();

            let video = Video {
                channel: channel.to_string(),
                title: title.to_string(),
                link: link.to_string(),
                published_at,
            };

            videos.push(video);
        }

        Ok(videos)
    }

    pub fn notification_text(&self, notifier: &Notifier) -> String {
        match notifier {
            Notifier::Log() => {
                format!("New video - {} {} {}", self.channel, self.title, self.link)
            }
            Notifier::Slack(_) => {
                format!(
                    "*New video - {}* <{}|{}>",
                    self.channel, self.link, self.title
                )
            }
            Notifier::Telegram => {
                todo!()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[tokio::test]
    async fn parse_videos_test() {
        let rss_data = fs::read_to_string("src/fixtures/yt_videos_data.xml").unwrap();
        let videos = Video::parse_rss(rss_data).unwrap();
        assert_eq!(videos.len(), 15);
    }
}
