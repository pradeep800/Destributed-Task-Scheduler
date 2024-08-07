use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use chrono::{DateTime, Utc};
use serde_json::json;
use std::sync::Arc;

use crate::state::AppState;

use super::AppError;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct CreateTaskBody {
    pub schedule_at: DateTime<Utc>,
    pub retry: i16,
}
pub async fn create_task(
    State(state): State<Arc<AppState>>,
    Json(task): Json<CreateTaskBody>,
) -> Result<impl IntoResponse, AppError> {
    if task.retry > 3 || task.retry < 0 {
        return Ok((
            StatusCode::BAD_REQUEST,
            Json(json!({"error":"tasks retry should be atleast 0 and max 3"})),
        ));
    }
    let today_time_in_second = Utc::now();
    if task.schedule_at < today_time_in_second {
        return Ok((
            StatusCode::BAD_REQUEST,
            Json(
                json!({"error":"schedule_at_in_second should be alteast greater than now (in server)"}),
            ),
        ));
    }
    let tracing_id = uuid::Uuid::new_v4().to_string();
    let res = sqlx::query!(
        "INSERT INTO Tasks (schedule_at, total_retry,tracing_id)
         VALUES ($1,$2,$3) RETURNING id",
        task.schedule_at,
        task.retry,
        tracing_id
    )
    .fetch_one(&state.db_pool.clone())
    .await
    .map_err(|e| AppError(anyhow::Error::new(e)))?;
    return Ok((
        StatusCode::OK,
        Json(json!({"id":res.id,"tracing_id":tracing_id})),
    ));
}
