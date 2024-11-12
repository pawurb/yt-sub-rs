use std::time::Duration;

use axum::{
    extract::Request,
    http::{HeaderValue, Uri},
    middleware::Next,
    response::{IntoResponse, Response},
};

use reqwest::StatusCode;
use time::UtcOffset;

use tower_http::{
    timeout::TimeoutLayer,
    trace::{self, TraceLayer},
};
use tracing::Level;
use tracing_subscriber::fmt::time::OffsetTime;

#[derive(Debug, PartialEq)]
enum Env {
    Development,
    Production,
    Test,
}

impl Env {
    fn current() -> Self {
        match std::env::var("ENV").expect("ENV is not set").as_str() {
            "development" => Self::Development,
            "production" => Self::Production,
            "test" => Self::Test,
            _ => panic!("Invalid ENV"),
        }
    }

    fn is_dev(&self) -> bool {
        self == &Self::Development
    }
}

pub fn logging() -> tower_http::trace::TraceLayer<
    tower_http::classify::SharedClassifier<tower_http::classify::ServerErrorsAsFailures>,
> {
    TraceLayer::new_for_http()
        .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
        .on_response(trace::DefaultOnResponse::new().level(Level::INFO))
}

pub fn timeout() -> TimeoutLayer {
    TimeoutLayer::new(Duration::from_secs(10))
}

pub async fn security_headers(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;

    response.headers_mut().insert(
        "X-Content-Type-Options",
        HeaderValue::from_static("nosniff"),
    );
    response
        .headers_mut()
        .insert("X-Frame-Options", HeaderValue::from_static("SAMEORIGIN"));
    response.headers_mut().insert(
        "referrer-policy",
        HeaderValue::from_static("no-referrer-when-downgrade"),
    );
    response.headers_mut().insert(
        "Strict-Transport-Security",
        HeaderValue::from_static("Strict-Transport-Security: max-age=31536000; includeSubDomains"),
    );

    response
}

pub async fn only_ssl(request: Request, next: Next) -> Response {
    let ssl = request
        .headers()
        .get("x-ssl")
        .and_then(|header| header.to_str().ok())
        == Some("true");

    if ssl || Env::current().is_dev() {
        next.run(request).await
    } else {
        let authority = request
            .headers()
            .get("host")
            .and_then(|header| header.to_str().ok())
            .unwrap_or("localhost");

        let uri = Uri::builder()
            .scheme("https")
            .authority(authority)
            .path_and_query(request.uri().path_and_query().unwrap().clone())
            .build()
            .unwrap();

        Response::builder()
            .status(StatusCode::MOVED_PERMANENTLY)
            .header("Location", uri.to_string())
            .body(axum::body::Body::empty())
            .unwrap()
            .into_response()
    }
}

pub fn init_logs(filename: &str) {
    let file_appender = tracing_appender::rolling::never("./", filename);

    let offset = UtcOffset::from_hms(1, 0, 0).expect("should get CET offset");
    let time_format =
        time::format_description::parse("[year]-[month]-[day]T[hour]:[minute]:[second]").unwrap();
    let timer = OffsetTime::new(offset, time_format);

    tracing_subscriber::fmt()
        .with_writer(file_appender)
        .with_timer(timer)
        .init();
}
