use crate::{
    configurations::Config,
    routes::heart_beat::{self, heart_beat},
};
use axum::{extract::Request, routing::get, serve::Serve, Router};
use sqlx::PgPool;
use std::{sync::Arc, time::Duration};
use tokio::net::TcpListener;
use tower_http::{classify::ServerErrorsFailureClass, trace::TraceLayer};
use tracing::{error, info_span, Span};
pub struct AppState {
    pub task_pool: PgPool,
    pub health_check_pool: PgPool,
    pub config: Config,
}

pub async fn get_server(listener: TcpListener, config: Config) -> Serve<Router, Router> {
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
    let share_state = Arc::new(AppState {
        task_pool: config.tasks_db.get_pool().await,
        health_check_pool: config.health_db.get_pool().await,
        config,
    });

    let router = Router::new()
        .route("/health-check", get(health_check))
        .route("/worker/heart-beat", get(heart_beat))
        .layer(tracing_layer)
        .with_state(share_state);

    axum::serve(listener, router)
}

async fn health_check() -> &'static str {
    "OK"
}
