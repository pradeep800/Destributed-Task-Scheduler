use std::sync::Arc;

use sqlx::PgPool;

pub struct AppState {
    pub pool: PgPool,
}

