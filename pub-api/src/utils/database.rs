use anyhow::Context;
use sqlx::{
    postgres::{PgPool, PgPoolOptions},
    Pool, Postgres,
};

use crate::configuration::Database;
pub async fn get_db_pool(database_config: Database) -> PgPool {
    let connecting_string = create_connecting_string(database_config);
    PgPoolOptions::new()
        .max_connections(5)
        .connect(&connecting_string)
        .await
        .context("could not able to connect database")
        .unwrap()
}
fn create_connecting_string(database_config: Database) -> String {
    format!(
        "postgres://{}:{}@localhost:5432/{}",
        database_config.database_user,
        database_config.database_password,
        database_config.database_db
    )
}
