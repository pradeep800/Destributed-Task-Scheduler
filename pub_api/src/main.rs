use anyhow::Context;
use pub_api::{
    configuration::get_configuration,
    startup::get_server,
    tracing::{get_subscriber, init_subscriber},
};

#[tokio::main]
async fn main() {
    let subscriber = get_subscriber(
        "pub_task_scheduler_api".to_string(),
        "info".to_string(),
        std::io::stdout,
    );
    let config = get_configuration();
    init_subscriber(subscriber);
    let db_pool = config.database.get_pool().await;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:4000")
        .await
        .unwrap();
    let _ = get_server(listener, db_pool, config)
        .await
        .context("can't spawan the server")
        .unwrap();
}
