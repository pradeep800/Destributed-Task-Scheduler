use common::database::Database;
use common::tracing::TRACING;
use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use tasks_producer::configuration::{get_configuration, Config};
async fn migrate_and_get_db(database: &mut Database) -> PgPool {
    let mut connection =
        PgConnection::connect(database.get_connecting_string_without_db().as_str())
            .await
            .expect("Failed to connect to Postgres");
    Lazy::force(&TRACING);
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
pub async fn spawn() -> Config {
    let mut configuration = get_configuration();
    let _ = migrate_and_get_db(&mut configuration.database).await;
    configuration
}
