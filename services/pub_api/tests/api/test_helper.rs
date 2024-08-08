use chrono::Utc;
use common::database::Database;
use common::tracing::{get_subscriber, init_subscriber};
use once_cell::sync::Lazy;
use pub_api::{configuration::get_configuration, startup::get_server};
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::future::IntoFuture;
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
    pub db_pool: PgPool,
}
pub async fn migrate_and_get_db(database: &mut Database) -> PgPool {
    let mut connection =
        PgConnection::connect(database.get_connecting_string_without_db().as_str())
            .await
            .expect("Failed to connect to Postgres");

    database.database_db += &uuid::Uuid::new_v4().to_string();
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, database.database_db).as_str())
        .await
        .expect("Failed to create database.");

    let connection_pool = database.get_pool().await;

    sqlx::migrate!("./../../crates/db/tasks/migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
}
pub async fn spawn() -> AppInfo {
    Lazy::force(&TRACING);
    let mut config = get_configuration();
    let db_pool = migrate_and_get_db(&mut config.tasks).await;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://localhost:{}", port);
    let server = get_server(listener, config).await;
    let _ = tokio::spawn(server.into_future());
    return AppInfo { address, db_pool };
}
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Task {
    pub id: i32,
    pub schedule_at: chrono::DateTime<Utc>,
    pub picked_at_by_producers: Vec<chrono::DateTime<Utc>>,
    pub picked_at_by_workers: Vec<chrono::DateTime<Utc>>,
    pub successful_at: Option<chrono::DateTime<Utc>>,
    pub failed_ats: Vec<chrono::DateTime<Utc>>,
    pub failed_reasons: Vec<String>,
    pub total_retry: i16,
    pub current_retry: i16,
    pub file_uploaded: bool,
    pub is_producible: bool,
    pub tracing_id: String,
}
impl Task {
    pub async fn create_task_in_db(&self, db_pool: &PgPool) -> Task {
        let inserted_task = sqlx::query_as!(
            Task,
            r#"
        INSERT INTO Tasks (
             id, schedule_at, picked_at_by_producers, picked_at_by_workers,
            successful_at, failed_ats, failed_reasons,
            total_retry, current_retry,
            file_uploaded, is_producible, tracing_id
        )
        VALUES (
            $1, $2, $3,
            $4, $5, $6,
            $7, $8,
            $9, $10, $11, $12
        )
        RETURNING *
        "#,
            self.id,
            self.schedule_at,
            &self.picked_at_by_producers,
            &self.picked_at_by_workers,
            self.successful_at,
            &self.failed_ats,
            &self.failed_reasons,
            self.total_retry,
            self.current_retry,
            self.file_uploaded,
            self.is_producible,
            self.tracing_id,
        )
        .fetch_one(db_pool)
        .await
        .unwrap();
        inserted_task
    }
}
