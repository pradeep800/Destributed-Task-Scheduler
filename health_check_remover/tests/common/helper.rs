use common::{database::Database, tracing::TRACING};
use health_check_remover::configuration::get_configuration;
use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection};
#[derive(Debug)]
pub struct AppInfo {
    pub database: Database,
}
pub async fn migrate_and_get_db(database: &mut Database) {
    let mut health_check_connection =
        PgConnection::connect(database.get_connecting_string_without_db().as_str())
            .await
            .expect("Failed to connect to Postgres");
    database.database_db += &uuid::Uuid::new_v4().to_string();
    health_check_connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, database.database_db).as_str())
        .await
        .expect("Failed to create database.");

    let health_check_pool = database.get_pool().await;
    sqlx::migrate!("./../db/health_checks/migrations")
        .run(&health_check_pool)
        .await
        .expect("Failed to migrate the database");
}
pub async fn spawn() -> AppInfo {
    Lazy::force(&TRACING);
    let mut database = get_configuration();
    migrate_and_get_db(&mut database).await;
    return AppInfo { database };
}
