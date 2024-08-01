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
        .cu_health_check_entries(claims.task_id, &claims.pod_name)
        .await
        .map_err(|e| {
            println!("{:?}", e);
            AppError::InternalServerError(e.into())
        })?;
    Ok(())
}
