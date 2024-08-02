use remove_health_check_entry::{configuration::get_configuration, process::process};

#[tokio::main]
async fn main() {
    let config = get_configuration();
    let pool = config.get_pool().await;

    process(&pool).await;
}
