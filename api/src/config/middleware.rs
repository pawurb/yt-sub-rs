use std::time::Duration;

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
