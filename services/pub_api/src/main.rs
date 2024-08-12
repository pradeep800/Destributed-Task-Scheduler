use common::tracing::{get_subscriber, init_subscriber};
use pub_api::{configuration::get_configuration, startup::get_server};

#[tokio::main]
async fn main() {
    let subscriber = get_subscriber("pub_api".to_string(), "info".to_string(), std::io::stdout);
    let config = get_configuration();
    init_subscriber(subscriber);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    let server = get_server(listener, config.clone()).await;
    server.await.unwrap();
}
