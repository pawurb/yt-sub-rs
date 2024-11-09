use eyre::Result;
use tracing::info;
use yt_sub_api::{
    config::{
        middleware::{init_logs, middleware},
        routes::app,
    },
    lite_helpers::{init_lite_db, sqlite_conn},
};

#[tokio::main]
async fn main() -> Result<()> {
    init_logs("server.log");

    init_lite_db(None).await.expect("Failed to init sqlite db");
    let conn = sqlite_conn(None)
        .await
        .expect("Failed to connect to sqlite db");
    let app = app(conn).await.layer(middleware());

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .unwrap();
    info!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
