use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use chrono::{DateTime, Timelike, Utc};
use serde_json::json;
use std::sync::Arc;

use tracing::error;

use crate::state::AppState;

#[derive(thiserror::Error, Debug)]
#[error("Internal Server Error")]
pub struct AppError(anyhow::Error);
impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        error!(error = ?self);
        (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", self.0)).into_response()
    }
}
#[derive(serde::Deserialize)]
pub struct Tasks {
    schedule_at_in_second: i32,
    retry: i16,
}
pub async fn create_task(
    State(state): State<Arc<AppState>>,
    Json(task): Json<Tasks>,
) -> Result<impl IntoResponse, AppError> {
    if task.retry > 3 || task.retry < 0 {
        return Ok((
            StatusCode::BAD_REQUEST,
            Json(json!({"error":"tasks retry should be atleast 1 and max 3"})),
        ));
    }
    let today_time_in_second = Utc::now().second() + 30;
    if task.schedule_at_in_second < today_time_in_second as i32 {
        return Ok((
            StatusCode::BAD_REQUEST,
            Json(
                json!({"error":"schedule_at_in_second should be alteast greater than now (in server)"}),
            ),
        ));
    }
    let res = sqlx::query!(
        "INSERT INTO Tasks (schedule_at_in_second, status, output, retry, created_at) 
         VALUES ($1,$2,$3,$4,$5) RETURNING id",
        task.schedule_at_in_second as i32,
        "ADDED",
        "",
        task.retry,
        Utc::now()
    )
    .fetch_one(&state.pool.clone())
    .await
    .map_err(|e| AppError(anyhow::Error::new(e)))?;
    return Ok((StatusCode::OK, Json(json!({"status":"ADDED","id":res.id}))));
}
#[derive(serde::Deserialize)]
pub struct Id {
    pub id: i32,
}
#[derive(serde::Serialize)]
pub struct Task {
    pub id: i32,
    pub schedule_at_in_second: i32,
    pub status: String,
    pub ouput: String,
    pub retry: i16,
    pub created_at: DateTime<Utc>,
}
pub async fn check_status(
    State(state): State<Arc<AppState>>,
    Json(body): Json<Id>,
) -> Result<impl IntoResponse, AppError> {
    let current_task = sqlx::query!("Select * from tasks where id=$1", body.id)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| AppError(anyhow::Error::new(e)))?;

    Ok(Json(Task {
        id: current_task.id,
        schedule_at_in_second: current_task.schedule_at_in_second,
        status: current_task.status,
        ouput: current_task.output,
        retry: current_task.retry,
        created_at: current_task.created_at,
    }))
}
