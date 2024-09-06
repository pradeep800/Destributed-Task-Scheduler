use crate::{error::AppError, startup::AppState};
use axum::{
    extract::{Extension, State},
    response::IntoResponse,
};
use common::jwt::Claims;
use health_checks::HealthCheckDb;
use std::sync::Arc;
use tracing::{info, info_span};
pub async fn heart_beat(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<impl IntoResponse, AppError> {
    let health_checks_db = HealthCheckDb::new(&state.health_check_pool);
    let span = info_span!("tracing={}", claims.tracing_id);
    let _guard = span.enter();
    health_checks_db
        .cu_health_check_entries(claims.task_id, &claims.pod_name)
        .await
        .map_err(|e| AppError::InternalServerError(e.into()))?;
    info!("helath check time is updated");
    Ok(())
}
