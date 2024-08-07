use common::tracing::{get_subscriber, init_subscriber};
use health_check_remover::{configuration::get_configuration, process::process};
#[tokio::main]
async fn main() {
    let subscriber = get_subscriber(
        "pub_task_scheduler_api".to_string(),
        "info".to_string(),
        std::io::stdout,
    );
    init_subscriber(subscriber);
    let config = get_configuration();
    let pool = config.get_pool().await;

    process(&pool).await;
}
