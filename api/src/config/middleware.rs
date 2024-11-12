use std::time::Duration;

use axum::{
    extract::Request,
    http::Uri,
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

pub async fn only_ssl(request: Request, next: Next) -> Response {
    let ssl = request
        .headers()
        .get("x-ssl")
        .and_then(|header| header.to_str().ok())
        == Some("true");

    if ssl {
        next.run(request).await
    } else {
        let uri = Uri::builder()
            .scheme("https")
            .authority(request.uri().authority().unwrap().clone())
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
