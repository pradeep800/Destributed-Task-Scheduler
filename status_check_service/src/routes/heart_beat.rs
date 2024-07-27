use std::sync::Arc;

use axum::{extract::State, http::HeaderMap, response::IntoResponse, routing::get, Json, Router};
use common::jwt::Jwt;
use health_checks::HealthCheckDb;
use reqwest::StatusCode;
use serde_json::json;
use tracing::error;

use crate::startup::AppState;
#[derive(thiserror::Error, Debug)]
#[error("Internal Server Error")]
pub enum AppError {
    Unauthorized,
    InternalServerError(anyhow::Error),
}
impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        error!(error = ?self);
        match self {
            AppError::InternalServerError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Internal Server Error"})),
            )
                .into_response(),
            AppError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "Please add authorization header"})),
            )
                .into_response(),
        }
    }
}
pub async fn heart_beat(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    let auth_header = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or(AppError::Unauthorized)?;
    let jwt = Jwt::new(state.config.jwt_secret.to_string());
    let claim = jwt
        .verify(auth_header)
        .map_err(|_e| AppError::Unauthorized)?;
    let health_checks_db = HealthCheckDb::new(&state.health_check_pool);

    health_checks_db
        .create_update_last_time_health_check(claim.task_id)
        .await
        .map_err(|e| {
            println!("{:?}", e);
            AppError::InternalServerError(e.into())
        })?;
    Ok(())
}
