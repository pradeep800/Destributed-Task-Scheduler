use common::tracing::{get_subscriber, init_subscriber};
use retry_and_failed_updater::{configuration::get_configuration, process::process};

#[tokio::main]
async fn main() {
    let subscriber = get_subscriber(
        "retry_and_failed_updater".to_string(),
        "info".to_string(),
        std::io::stdout,
    );
    init_subscriber(subscriber);
    let config = get_configuration();
    process(&config).await;
}
