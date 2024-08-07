use tasks_producer::{configuration::get_configuration, producer::producer};

use common::tracing::{get_subscriber, init_subscriber};
#[tokio::main]
async fn main() {
    let subscriber = get_subscriber(
        "pub_task_scheduler_api".to_string(),
        "info".to_string(),
        std::io::stdout,
    );
    init_subscriber(subscriber);
    let config = get_configuration();
    producer(&config).await;
}
