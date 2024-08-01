use common::database::Database;
use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use task_producer::configuration::{get_configuration, Config};
async fn migrate_and_get_db(database: &mut Database) -> PgPool {
    let mut connection =
        PgConnection::connect(database.get_connecting_string_without_db().as_str())
            .await
            .expect("Failed to connect to Postgres");
    Lazy::force(&TRACING);
    database.database_db += &uuid::Uuid::new_v4().to_string();
    println!("{:?}", database);
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, database.database_db).as_str())
        .await
        .expect("Failed to create database.");

    let connection_pool = database.get_pool().await;

    sqlx::migrate!("./../db/tasks/migrations")
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
