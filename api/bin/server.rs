use axum::middleware::from_fn;
use eyre::Result;
use std::time::Duration;
use tokio::net::TcpListener;
use tower_http::{
    catch_panic::CatchPanicLayer, compression::CompressionLayer, timeout::TimeoutLayer,
};
use tracing::info;
use yt_sub_api::{
    config::{middleware, routes::app},
    lite_helpers::{init_lite_db, sqlite_conn},
};

#[tokio::main]
async fn main() -> Result<()> {
    match run().await {
        Ok(_) => Ok(()),
        Err(e) => {
            tracing::error!("{:?}", e);
            Err(e)
        }
    }
}

async fn run() -> Result<()> {
    middleware::init_logs("server.log");

    init_lite_db(None).await?;
    let conn = sqlite_conn(None).await?;
    let app = app(conn)
        .await
        .layer(middleware::logging())
        .layer(from_fn(middleware::only_ssl))
        .layer(TimeoutLayer::new(Duration::from_secs(10)))
        .layer(CompressionLayer::new())
        .layer(CatchPanicLayer::new())
        .layer(from_fn(middleware::security_headers));

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());

    if TcpListener::bind(format!("0.0.0.0:{port}")).await.is_err() {
        eyre::bail!("Port {} is already in use", port);
    }

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;

    info!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
