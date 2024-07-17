use sqlx::postgres::PgPool;
fn get_pool(database_config: Database) {
    PgPool::connect_lazy(url)
}
