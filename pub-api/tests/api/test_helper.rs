use std::future::IntoFuture;

use once_cell::sync::Lazy;
use pub_api::{
    configuration::{get_configuration, Database},
    startup::get_server,
    utils::{
        database::{create_connection_without_db, get_db_pool},
        tracing::{get_subscriber, init_subscriber},
    },
};
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    query, ConnectOptions, Connection, PgConnection, PgPool,
};
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
    let postfix = uuid::Uuid::new_v4().to_string();
    database.database_db += &postfix;
    let db_pool = get_db_pool(database).await;
    let connection_string = create_connection_without_db(database);
    let pgConnection = PgConnectOptions::new()
        .host("localhost")
        .username(&database.database_user)
        .port(5432);
    let mut connection = PgConnection::connect_with(&pgConnection)
        .await
        .expect("Failed to connect to Postgres");
    query!(format!(r#"CREATE DATABASE "{}";"#, database.database_db).
    connection
        .execute(&*))
    .await
    .expect("Failed to create database.");
    let db_pool = get_db_pool(database).await;
    let _ = sqlx::migrate!("./migrations").run(&db_pool).await;
    db_pool
}
pub async fn spawn() -> AppInfo {
    let mut config = get_configuration();
    Lazy::force(&TRACING);
    let db_pool = migrate_and_get_db(&mut config.database).await;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://localhost:{}", port);
    let server = get_server(listener, db_pool.clone());
    let _ = tokio::spawn(server.into_future());
    return AppInfo { address, db_pool };
}
