use std::time::Duration;

use time::UtcOffset;

use tower::{
    layer::util::{Identity, Stack},
    ServiceBuilder,
};

use tower_http::{
    classify::{ServerErrorsAsFailures, SharedClassifier},
    timeout::TimeoutLayer,
    trace::{self, TraceLayer},
};
use tracing::Level;
use tracing_subscriber::fmt::time::OffsetTime;

type Middleware = ServiceBuilder<
    Stack<TimeoutLayer, Stack<TraceLayer<SharedClassifier<ServerErrorsAsFailures>>, Identity>>,
>;

pub fn middleware() -> Middleware {
    ServiceBuilder::new()
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        )
        .layer(TimeoutLayer::new(Duration::from_secs(10)))
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
