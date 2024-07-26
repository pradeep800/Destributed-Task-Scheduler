use remove_health_check_entry::configuration::get_configuration;

#[tokio::main]
async fn main() {
    let config = get_configuration();
    let pool = config.get_pool().await;
    loop {
        sqlx::query!("DELETE FROM health_checks WHERE task_completed = true")
            .execute(&pool)
            .await
            .unwrap();
    }
}
