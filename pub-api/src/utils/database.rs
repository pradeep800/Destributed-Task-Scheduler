use anyhow::Context;
use sqlx::postgres::{PgPool, PgPoolOptions};

use crate::configuration::Database;
pub async fn get_db_pool(database_config: &mut Database) -> PgPool {
    let connecting_string = create_connecting_string(database_config);
    PgPoolOptions::new()
        .max_connections(5)
        .connect(&connecting_string)
        .await
        .context("could not able to connect database")
        .unwrap()
}
fn create_connecting_string(database_config: &mut Database) -> String {
    format!(
        "postgres://{}:{}@localhost:5432/{}",
        database_config.database_user,
        database_config.database_password,
        database_config.database_db
    )
}
pub fn create_connection_without_db(database_config: &mut Database) -> String {
    format!(
        "postgres://{}:{}@localhost:5432",
        database_config.database_user, database_config.database_password,
    )
}
