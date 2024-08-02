use retry_and_failed_updater_service::{
    configuration::get_configuration, process::process, tracing::get_subscriber,
};

#[tokio::main]
async fn main() {
    let subscriber = get_subscriber(
        "Retry and failed updater".to_string(),
        "info".to_string(),
        std::io::stdout,
    );
    let config = get_configuration();
    process(&config).await;
}
