use axum::{extract::State, http::StatusCode, response::IntoResponse};
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

#[derive(sqlx::Type)]
#[sqlx(type_name = "status")]
#[sqlx(rename_all = "UPPERCASE")]
enum Status {
    Completed,
    Processing,
    Failed,
    Added,
}
#[derive(sqlx::FromRow)]
struct Tasks {
    id: String,
    schedule_at_in_second: i32,
    status: Status,
    output: String,
    retry: i16,
    created_at: chrono::DateTime<chrono::Local>,
}
pub async fn create_task(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let query = sqlx::query_as!(Tasks, "select * from tasks")
        .fetch_one(&state.pool)
        .await
        .unwrap();
    Ok((StatusCode::OK, "hello"))
}
pub async fn check_status(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    Ok((StatusCode::OK, "hello"))
}
