use reqwest::Client;
use serde_json::Value;

#[tokio::test]
async fn test_get_channel_data() {
    let client = Client::new();
    let res = client
        .get("https://yt-sub-api.apki.workers.dev/channel_data")
        .header("x-handle", "@ManOfRecaps")
        .send()
        .await
        .expect("Failed to send request");
    assert_eq!(res.status(), 200);

    let res_json: Value = res.json().await.expect("Failed to parse JSON");
    assert_eq!(res_json["channel_id"], "UCNCTxLZ3EKKry-oWgLlsYsw");
    assert_eq!(res_json["channel_name"], "Man of Recaps");
}

#[tokio::test]
async fn test_get_invalid_channel_data() {
    let client = Client::new();
    let res = client
        .get("https://yt-sub-api.apki.workers.dev/channel_data")
        .header("x-handle", "@kljjfadslufd")
        .send()
        .await
        .expect("Failed to send request");
    assert_eq!(res.status(), 404);
}
