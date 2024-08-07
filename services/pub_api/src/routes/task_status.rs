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

#[derive(serde::Serialize, serde::Deserialize)]
pub struct GetAllTaskStatusResponse {
    pub id: i32,
    pub schedule_at: DateTime<Utc>,
    pub status: String,
    pub total_retry: i16,
    pub current_retry: i16,
    pub tracing_id: String,
}
pub async fn get_all_task_status(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let all_task = sqlx::query!("Select * from tasks order by id")
        .fetch_all(&state.db_pool)
        .await
        .map_err(|e| AppError(anyhow::Error::new(e)))?;

    let mut response: Vec<GetAllTaskStatusResponse> = Vec::new();
    for current_task in all_task {
        let status = get_status(&GetStatusParms {
            successful_at: &current_task.successful_at,
            picked_at_by_workers: &current_task.picked_at_by_workers,
            failed_ats: &current_task.failed_ats,
            total_retry: current_task.total_retry,
            current_retry: current_task.current_retry,
            file_uploaded: current_task.file_uploaded,
        });

        let task = GetAllTaskStatusResponse {
            id: current_task.id,
            schedule_at: current_task.schedule_at,
            status,
            total_retry: current_task.total_retry,
            current_retry: current_task.current_retry,
            tracing_id: current_task.tracing_id,
        };
        response.push(task);
    }
    Ok((StatusCode::OK, Json(json!(response))))
}
pub struct GetStatusParms<'a> {
    pub successful_at: &'a Option<DateTime<Utc>>,
    pub picked_at_by_workers: &'a Vec<DateTime<Utc>>,
    pub failed_ats: &'a Vec<DateTime<Utc>>,
    pub total_retry: i16,
    pub current_retry: i16,
    pub file_uploaded: bool,
}
pub fn get_status(task: &GetStatusParms) -> String {
    let mut status = "PROCESSING".to_string();
    if !task.file_uploaded {
        status = "FILE_NOT_UPLOADED".to_string();
    }
    // if successful_at is added that mean our task got successfully finished
    else if !task.successful_at.is_none() {
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
            file_uploaded: true,
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
            file_uploaded: true,
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
            file_uploaded: true,
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
            file_uploaded: true,
            total_retry: 3,
        });
        assert_eq!(status, "PROCESSING");
    }
    #[test]
    fn get_file_not_uploaded() {
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
            file_uploaded: false,
            total_retry: 3,
        });
        assert_eq!(status, "FILE_NOT_UPLOADED");
    }
}
