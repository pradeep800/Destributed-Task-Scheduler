use status_check_service::{configurations::get_configuration, startup::get_server};
#[tokio::main]
async fn main() {
    let config = get_configuration();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:4000")
        .await
        .unwrap();
    let server = get_server(listener, config).await;
    server.await.unwrap();
}
