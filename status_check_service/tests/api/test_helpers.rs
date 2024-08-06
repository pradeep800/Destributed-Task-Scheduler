use chrono::{Duration, Utc};
use common::tracing::{get_subscriber, init_subscriber};
use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection};
use status_check_service::{
    configurations::{get_configuration, Config},
    startup::get_server,
};
use std::future::IntoFuture;
use tasks::Task;
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
    pub address: String,
    pub config: Config,
}
pub async fn migrate_and_get_db(config: &mut Config) {
    //start migration of task pool
    let mut task_db_connection =
        PgConnection::connect(config.tasks.get_connecting_string_without_db().as_str())
            .await
            .expect("Failed to connect to Postgres");

    config.tasks.database_db += &uuid::Uuid::new_v4().to_string();
    task_db_connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.tasks.database_db).as_str())
        .await
        .expect("Failed to create database.");

    let task_pool = config.tasks.get_pool().await;

    sqlx::migrate!("./../db/tasks/migrations")
        .run(&task_pool)
        .await
        .expect("Failed to migrate the database");

    //start migration of health_check_pool
    let mut health_check_connection = PgConnection::connect(
        config
            .health_check
            .get_connecting_string_without_db()
            .as_str(),
    )
    .await
    .expect("Failed to connect to Postgres");
    config.health_check.database_db += &uuid::Uuid::new_v4().to_string();
    health_check_connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.health_check.database_db).as_str())
        .await
        .expect("Failed to create database.");

    let health_check_pool = config.health_check.get_pool().await;
    sqlx::migrate!("./../db/health_checks/migrations")
        .run(&health_check_pool)
        .await
        .expect("Failed to migrate the database");
}
pub async fn spawn() -> AppInfo {
    Lazy::force(&TRACING);
    let mut config = get_configuration();
    migrate_and_get_db(&mut config).await;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://localhost:{}", port);
    let server = get_server(listener, config.clone()).await;
    let _ = tokio::spawn(server.into_future());
    return AppInfo { address, config };
}
pub fn generate_random_processing_task() -> Task {
    let mut failed_reasons = Vec::<String>::new();

    let mut failed_ats = Vec::<chrono::DateTime<Utc>>::new();
    let mut picked_at_by_producers = Vec::<chrono::DateTime<Utc>>::new();

    let mut picked_at_by_workers = Vec::<chrono::DateTime<Utc>>::new();
    for i in 0..3 {
        failed_reasons.push(format!("Failed reason {}", i + 1));
        failed_ats.push(Utc::now());
        picked_at_by_producers.push(Utc::now());
        picked_at_by_workers.push(Utc::now());
    }
    failed_ats.pop();
    failed_reasons.pop();
    let new_task = Task {
        id: 1,
        schedule_at: Utc::now() + Duration::minutes(1),
        picked_at_by_workers,
        picked_at_by_producers,
        successful_at: None,
        failed_ats,
        failed_reasons,
        total_retry: 3,
        current_retry: 2,
        file_uploaded: true,
        is_producible: true,
        tracing_id: uuid::Uuid::new_v4().to_string(),
    };
    new_task
}
