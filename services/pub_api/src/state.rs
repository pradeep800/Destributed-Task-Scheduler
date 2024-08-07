use sqlx::PgPool;
use std::sync::Arc;

use crate::configuration::Config;
pub struct AppState {
    pub db_pool: PgPool,
    pub config: Arc<Config>,
}
