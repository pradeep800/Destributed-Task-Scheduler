use crate::{
    routes::{check_status, create_sign_url, create_task, get_all_task_status, upload_status},
    state::AppState,
};
use axum::http::Method;
use axum::{
    extract::Request,
    routing::{get, post},
    serve::Serve,
    Router,
};
use std::{sync::Arc, time::Duration};
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use tower_http::{classify::ServerErrorsFailureClass, trace::TraceLayer};
use tracing::{error, info_span, Span};
pub async fn get_server(
    listener: TcpListener,
    config: crate::configuration::Config,
) -> Serve<Router, Router> {
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
        db_pool: config.database.get_pool().await,
        config: Arc::new(config),
    });
    let cors = CorsLayer::new()
        .allow_methods(vec![Method::GET, Method::POST])
        .allow_origin(Any)
        .allow_headers(Any);
    let router = Router::new()
        .route("/health-check", get(health_check))
        .route("/task/create", post(create_task))
        .route("/task/all/status", get(get_all_task_status))
        .route("/signurl/create", post(create_sign_url))
        .route("/file/status", post(upload_status))
        .route("/task/status", post(check_status))
        .layer(cors)
        .layer(tracing_layer)
        .with_state(share_state);

    axum::serve(listener, router)
}

async fn health_check() -> &'static str {
    "OK"
}
