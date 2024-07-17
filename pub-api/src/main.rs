use axum::{
    extract::Request,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use config::{Config, File, FileFormat};
use pub_api::{routes::tasks::create_task, utils::tracing::*};
use std::time::Duration;
use tower_http::{classify::ServerErrorsFailureClass, trace::TraceLayer};
use tracing::{debug, debug_span, error, event, info, info_span, span, Level, Span};

async fn health_check() -> &'static str {
    "OK"
}
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Database {
    #[serde(alias = "name")]
    database_user: String,

    #[serde(alias = "DATBASE_DB")]
    database_db: String,

    #[serde(alias = "DATBASE_PASSWORD")]
    database_password: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct EnvVariable {
    database: Database,
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
    let builder = Config::builder().add_source(File::new("env.yaml", FileFormat::Yaml));
    let config = builder.build().unwrap();
    let env: EnvVariable = config.try_deserialize().unwrap();
    let subscriber = get_subscriber("pub-api".into(), "debug".to_string(), std::io::stdout);

    init_subscriber(subscriber);

    let app = Router::new()
        .route("/health-check", get(health_check))
        .route("/task/create", post(create_task))
        .layer(tracing_layer);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
