use reqwest::Client;
use serde_json::Value;

#[tokio::test]
async fn test_get_channel_data() {
    let client = Client::new();
    let res = client
        .get("https://yt-sub-api.apki.workers.dev/channel_data/@ManOfRecaps")
        .send()
        .await
        .expect("Failed to send request");
    assert_eq!(res.status(), 200);

    let res_json: Value = res.json().await.expect("Failed to parse JSON");
    assert_eq!(res_json["channel_id"], "UCNCTxLZ3EKKry-oWgLlsYsw");
    assert_eq!(res_json["channel_name"], "Man of Recaps");
}

#[tokio::test]
async fn test_invalid_channel_data() {
    let client = Client::new();
    let res = client
        .get("https://yt-sub-api.apki.workers.dev/channel_data/@kljjfadslufd")
        .send()
        .await
        .expect("Failed to send request");
    assert_eq!(res.status(), 404);
}

#[tokio::test]
async fn test_missing_handle() {
    let client = Client::new();
    let res = client
        .get("https://yt-sub-api.apki.workers.dev/channel_data")
        .send()
        .await
        .expect("Failed to send request");
    assert_eq!(res.status(), 404);
}

#[tokio::test]
async fn test_failed_register() {
    let client = Client::new();
    let res = client
        .post("https://yt-sub-api.apki.workers.dev/register")
        .send()
        .await
        .expect("Failed to send request");
    assert_eq!(res.status(), 400);
}
