use super::{get_status, AppError, GetStatusParms};
use crate::state::AppState;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use chrono::{DateTime, Utc};
use serde_json::json;
use std::sync::Arc;

#[derive(serde::Deserialize)]
pub struct Id {
    pub id: i32,
}
#[derive(serde::Serialize, serde::Deserialize)]
pub struct TaskStatusResponse {
    pub id: i32,
    pub schedule_at: DateTime<Utc>,
    pub status: String,
    pub total_retry: i16,
    pub currently_retrying_at: i16,
    pub tracing_id: String,
}
pub async fn check_status(
    State(state): State<Arc<AppState>>,
    Json(body): Json<Id>,
) -> Result<impl IntoResponse, AppError> {
    let current_task = sqlx::query!("Select * from tasks where id=$1", body.id)
        .fetch_one(&state.db_pool)
        .await
        .map_err(|e| AppError(anyhow::Error::new(e)))?;
    if current_task.file_uploaded == false {
        return Ok((
            StatusCode::BAD_REQUEST,
            Json(json!({"error":"please upload executable file"})),
        ));
    }

    let status = get_status(&GetStatusParms {
        successful_at: &current_task.successful_at,
        picked_at_by_workers: &current_task.picked_at_by_workers,
        failed_ats: &current_task.failed_ats,
        total_retry: current_task.total_retry,
        current_retry: current_task.current_retry,
        file_uploaded: current_task.file_uploaded,
    });
    Ok((
        StatusCode::OK,
        Json(json!(TaskStatusResponse {
            id: current_task.id,
            schedule_at: current_task.schedule_at,
            status,
            total_retry: current_task.total_retry,
            currently_retrying_at: current_task.current_retry,
            tracing_id: current_task.tracing_id,
        })),
    ))
}
