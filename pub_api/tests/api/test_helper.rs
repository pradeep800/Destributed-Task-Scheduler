use std::{env, future::IntoFuture};

use common::database::Database;
use once_cell::sync::Lazy;
use pub_api::{
    configuration::get_configuration,
    startup::get_server,
    tracing::{get_subscriber, init_subscriber},
};
use sqlx::{Connection, Executor, PgConnection, PgPool};
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

    // Migrate database
    let connection_pool = database.get_pool().await;
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
}
pub async fn spawn() -> AppInfo {
    Lazy::force(&TRACING);
    let mut database = get_configuration();
    let db_pool = migrate_and_get_db(&mut database).await;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://localhost:{}", port);
    let server = get_server(listener, db_pool.clone());
    let _ = tokio::spawn(server.into_future());
    return AppInfo { address, db_pool };
}
