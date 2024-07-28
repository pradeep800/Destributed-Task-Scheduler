use axum::{
    extract::{Extension, Json, State},
    response::IntoResponse,
};
use std::sync::Arc;
use tracing_subscriber::fmt::format;

use crate::{error::AppError, startup::AppState};
use common::jwt::Claims;

struct Body {
    status: String,
}

pub async fn update_status(
    State(state): State<Arc<AppState>>,
    Json(body): Json<Body>,
    Extension(claims): Extension<Claims>,
) -> Result<impl IntoResponse, AppError> {
    if body.status == "SUCCESS" {
    
        sqlx::query!("  )
    } else if body.status == "FAILED" {
    }

    Err(AppError(
        format!("{} status is not possible", body.status).into(),
    ))
}
