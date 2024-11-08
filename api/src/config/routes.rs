use std::sync::Arc;

use crate::controllers;
use axum::{
    body::Body,
    extract::Path,
    http::{HeaderMap, Response, StatusCode},
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use sqlx::SqlitePool;
use tower_http::trace::{self, TraceLayer};
use tracing::Level;
use yt_sub_core::UserSettings;

pub async fn app(conn: Arc<SqlitePool>) -> Router {
    Router::new()
        .route("/channel_data/:handle", get({
          move |handle: Path<String>| async move {
            controllers::channels::show(handle.to_string()).await
          }
        }))
        .route(
            "/account",
            post({
                let conn = conn.clone();
                |Json(settings): Json<UserSettings>| async move {
                    controllers::account::create(settings, &conn).await
                }
            }),
        )
        .route(
            "/account",
            delete({
                let conn = conn.clone();
                |headers: HeaderMap| async move {
                  controllers::account::delete(headers, &conn).await
                }
            }),
        )
        .route(
            "/account",
            put({
                let conn = conn.clone();
                move |Json(settings): Json<UserSettings>| async move {
                    controllers::account::update(settings, &conn).await
                }
            }),
        )
        .route("/uptime", get(
          {
            || async move {
                "OK".into_response()
            }
          }
        ))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        )
}

pub fn invalid_req(reason: &str) -> Response<Body> {
    (StatusCode::BAD_REQUEST, reason.to_string()).into_response()
}

#[cfg(test)]
mod tests {
    use crate::lite_helpers::tests::setup_test_db;

    use super::*;
    use axum::http::Request;
    use eyre::Result;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    #[tokio::test]
    async fn uptime_test() -> Result<()> {
        let (conn, _cl) = setup_test_db().await;
        let app = app(conn).await;

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/uptime")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(body, "OK");
        Ok(())
    }
}
