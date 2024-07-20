use anyhow::Context;
use pub_api::{
    configuration::get_configuration,
    startup::get_server,
    utils::{database::get_db_pool, tracing::*},
};

#[tokio::main]
async fn main() {
    let subscriber = get_subscriber(
        "pub_task_scheduler_api".to_string(),
        "info".to_string(),
        std::io::stdout,
    );
    init_subscriber(subscriber);
    let mut config = get_configuration();

    let db_pool = get_db_pool(&mut config.database).await;

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    let _ = get_server(listener, db_pool)
        .await
        .context("can't spawan the server")
        .unwrap();
}
