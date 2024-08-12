use common::tracing::{get_subscriber, init_subscriber};
use status_check::{configurations::get_configuration, startup::get_server};
#[tokio::main]
async fn main() {
    let subscriber = get_subscriber(
        "status_check".to_string(),
        "info".to_string(),
        std::io::stdout,
    );
    init_subscriber(subscriber);
    let config = get_configuration();

    let listener = tokio::net::TcpListener::bind("0.0.0.0:4000").await.unwrap();
    let server = get_server(listener, config).await;
    server.await.unwrap();
}
