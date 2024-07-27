use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use chrono::{DateTime, Utc};
use serde_json::json;
use std::sync::Arc;

use tracing::error;

use crate::state::AppState;

#[derive(thiserror::Error, Debug)]
#[error("Internal Server Error")]
pub struct AppError(pub anyhow::Error);
impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        error!(error = ?self);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error":"Internal Server Error"})),
        )
            .into_response()
    }
}

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
struct GetStatusParms<'a> {
    successful_at: &'a Option<DateTime<Utc>>,
    picked_at_by_workers: &'a Vec<DateTime<Utc>>,
    failed_ats: &'a Vec<DateTime<Utc>>,
    total_retry: i16,
    current_retry: i16,
}
fn get_status(task: &GetStatusParms) -> String {
    let mut status = "PROCESSING".to_string();
    // if successful_at is added that mean our task got successfully finished
    if !task.successful_at.is_none() {
        status = "SUCCESS".to_string();
    }
    // if it never got picked by any worker that mean is only got scheduled
    else if task.picked_at_by_workers.len() == 0 {
        status = "SCHEDULED".to_string();
    }
    // if we used our all retry and all failure record are there (total_retry + first_try) it mean
    // it got failed
    else if task.total_retry == task.current_retry
        && task.failed_ats.len() as i16 == task.total_retry + 1
    {
        status = "FAILED".to_string();
    }
    // if  we have retry left or all failure record are not there it mean our task is still getting
    // processed
    status
}
#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn get_status_check_for_successful_status() {
        let status = get_status(&GetStatusParms {
            successful_at: &Some(Utc::now()),
            picked_at_by_workers: &Vec::new(),
            failed_ats: &Vec::new(),
            current_retry: 0,
            total_retry: 3,
        });
        assert_eq!(status, "SUCCESS");
    }
    #[test]
    fn get_status_check_for_failed_status() {
        let mut failed_at = Vec::new();
        let mut picked_by_worker = Vec::new();
        picked_by_worker.push(Utc::now());
        picked_by_worker.push(Utc::now());
        picked_by_worker.push(Utc::now());
        picked_by_worker.push(Utc::now());
        failed_at.push(Utc::now());
        failed_at.push(Utc::now());
        failed_at.push(Utc::now());
        failed_at.push(Utc::now());
        let status = get_status(&GetStatusParms {
            successful_at: &None,
            picked_at_by_workers: &picked_by_worker,
            failed_ats: &failed_at,
            current_retry: 3,
            total_retry: 3,
        });
        assert_eq!(status, "FAILED");
    }
    #[test]

    fn get_status_check_for_scheduled_status() {
        let status = get_status(&GetStatusParms {
            successful_at: &None,
            picked_at_by_workers: &Vec::new(),
            failed_ats: &Vec::new(),
            current_retry: 0,
            total_retry: 3,
        });
        assert_eq!(status, "SCHEDULED");
    }
    #[test]
    fn get_status_check_for_processing_status() {
        let mut failed_at = Vec::new();
        let mut picked_at_worker = Vec::new();
        picked_at_worker.push(Utc::now());
        picked_at_worker.push(Utc::now());
        picked_at_worker.push(Utc::now());
        failed_at.push(Utc::now());
        failed_at.push(Utc::now());
        failed_at.push(Utc::now());
        let status = get_status(&GetStatusParms {
            successful_at: &None,
            picked_at_by_workers: &picked_at_worker,
            failed_ats: &failed_at,
            current_retry: 2,
            total_retry: 3,
        });
        assert_eq!(status, "PROCESSING");
    }
}
