use std::sync::Arc;

use crate::controllers;
use axum::{
    body::Body,
    http::{Response, StatusCode},
    response::IntoResponse,
    routing::{delete, get, post, put},
    Router,
};
use sqlx::SqlitePool;

#[derive(Clone, Debug)]
pub struct AppState {
    pub conn: Arc<SqlitePool>,
}

pub async fn app(conn: Arc<SqlitePool>) -> Router {
    let state = AppState { conn };

    Router::new()
        .route("/channel_data/:handle", get(controllers::channels::show))
        .route("/account", post(controllers::account::create))
        .route("/account", delete(controllers::account::delete))
        .route("/account", put(controllers::account::update))
        .route("/uptime", get(|| async move { "OK".into_response() }))
        .with_state(state)
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
