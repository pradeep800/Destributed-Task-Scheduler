use once_cell::sync::Lazy;
use retry_and_failed_updater_service::{
    configuration::{get_configuration, Config},
    tracing::{get_subscriber, init_subscriber},
};
use sqlx::{Connection, Executor, PgConnection};
static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    };
});
#[derive(Debug)]
pub struct AppInfo {
    pub config: Config,
}
pub async fn migrate_and_get_db(config: &mut Config) {
    //start migration of task pool

    let mut task_db_connection =
        PgConnection::connect(config.tasks_db.get_connecting_string_without_db().as_str())
            .await
            .expect("Failed to connect to Postgres");

    config.tasks_db.database_db += &uuid::Uuid::new_v4().to_string();
    task_db_connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.tasks_db.database_db).as_str())
        .await
        .expect("Failed to create database.");

    let task_pool = config.tasks_db.get_pool().await;

    sqlx::migrate!("./../db/tasks/migrations")
        .run(&task_pool)
        .await
        .expect("Failed to migrate the database");

    //start migration of health_check_pool
    let mut health_check_connection =
        PgConnection::connect(config.health_db.get_connecting_string_without_db().as_str())
            .await
            .expect("Failed to connect to Postgres");
    config.health_db.database_db += &uuid::Uuid::new_v4().to_string();
    health_check_connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.health_db.database_db).as_str())
        .await
        .expect("Failed to create database.");

    let health_check_pool = config.health_db.get_pool().await;
    sqlx::migrate!("./../db/health_checks/migrations")
        .run(&health_check_pool)
        .await
        .expect("Failed to migrate the database");
}
pub async fn spawn() -> AppInfo {
    Lazy::force(&TRACING);
    let mut config = get_configuration();
    migrate_and_get_db(&mut config).await;
    return AppInfo { config };
}
