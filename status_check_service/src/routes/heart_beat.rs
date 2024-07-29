use std::sync::Arc;

use axum::{
    extract::{Extension, State},
    response::IntoResponse,
};
use common::jwt::Claims;
use health_checks::HealthCheckDb;

use crate::{error::AppError, startup::AppState};
pub async fn heart_beat(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<impl IntoResponse, AppError> {
    let health_checks_db = HealthCheckDb::new(&state.health_check_pool);

    health_checks_db
        .create_update_last_time_health_check(claims.task_id)
        .await
        .map_err(|e| {
            println!("{:?}", e);
            AppError::InternalServerError(e.into())
        })?;
    Ok(())
}
