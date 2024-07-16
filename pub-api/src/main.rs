use std::time::Duration;

use axum::{extract::Request, routing::get, Router};
use pub_api::utils::tracing::*;
use tower_http::{classify::ServerErrorsFailureClass, trace::TraceLayer};
use tracing::{error, event, info_span, span, Level, Span};

async fn health_check() -> &'static str {
    "OK"
}

#[tokio::main]
async fn main() {
    let tracing_layer = TraceLayer::new_for_http()
        .make_span_with(|request: &Request<_>| {
            let request_id = uuid::Uuid::new_v4();
            info_span!(
                "http_request",
                method = ?request.method(),
                matched_path = %request.uri().path(),
                %request_id
            )
        })
        .on_failure(
            |err: ServerErrorsFailureClass, _latency: Duration, _span: &Span| {
                error!(error = %err);
            },
        );

    let subscriber = get_subscriber("pub-api".into(), "info".to_string(), std::io::stdout);

    init_subscriber(subscriber);
    let app = Router::new()
        .route("/health-check", get(health_check))
        .layer(tracing_layer);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
