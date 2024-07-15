use axum::{routing::get, Router};
async fn health_check() -> &'static str {
    "OK"
}
#[tokio::main]
async fn main() {
    let app = Router::new().route("/health-check", get(health_check));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
