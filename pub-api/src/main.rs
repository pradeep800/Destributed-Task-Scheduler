use axum::{
    extract::Request,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use config::{Config, File, FileFormat};
use pub_api::state::AppState;
use pub_api::{
    configuration::get_configuration,
    routes::tasks::{check_status, create_task},
    utils::{database::get_db_pool, tracing::*},
};
use sqlx::{PgPool, Postgres};
use std::{sync::Arc, time::Duration};
use tower_http::{classify::ServerErrorsFailureClass, trace::TraceLayer};
use tracing::{debug, debug_span, error, event, info, info_span, span, Level, Span};
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
    let config = get_configuration();

    let db_pool = get_db_pool(config.database).await;

    let subscriber = get_subscriber("pub-api".into(), "debug".to_string(), std::io::stdout);

    init_subscriber(subscriber);
    let share_state = Arc::new(AppState { pool: db_pool });
    let app = Router::new()
        .route("/health-check", get(health_check))
        .route("/task/create", post(create_task))
        .route("/task/status", post(check_status))
        .layer(tracing_layer)
        .with_state(share_state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
async fn health_check() -> &'static str {
    "OK"
}
