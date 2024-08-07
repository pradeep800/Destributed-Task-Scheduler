use axum::{response::IntoResponse, Json};
use reqwest::StatusCode;
use serde_json::json;
use tracing::error;
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
