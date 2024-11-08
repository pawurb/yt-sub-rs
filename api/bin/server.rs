use eyre::Result;
use tracing::info;
use yt_sub_api::{
    config::routes::app,
    lite_helpers::{init_lite_db, sqlite_conn},
};

#[tokio::main]
async fn main() -> Result<()> {
    let file_appender = tracing_appender::rolling::never("./", "server.log");

    tracing_subscriber::fmt().with_writer(file_appender).init();

    init_lite_db(None).await.expect("Failed to init sqlite db");
    let conn = sqlite_conn(None)
        .await
        .expect("Failed to connect to sqlite db");
    let app = app(conn).await;

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .unwrap();
    info!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
