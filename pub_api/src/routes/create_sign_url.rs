use crate::state::AppState;
use anyhow::Result;
use aws_sdk_s3::presigning::PresigningConfig;
use aws_sdk_s3::types::ObjectCannedAcl;
use axum::Json;
use axum::{extract::State, response::IntoResponse};
use reqwest::StatusCode;
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;

use super::AppError;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct SignUrlBody {
    pub id: i32,
    pub executable_size: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
pub struct SignUrlResponse {
    pub presigned_url: String,
}
pub async fn create_sign_url(
    State(app_state): State<Arc<AppState>>,
    Json(body): Json<SignUrlBody>,
) -> Result<impl IntoResponse, AppError> {
    let twenty_five_mb = 50 * 1024 * 1024;
    if twenty_five_mb < body.executable_size {
        return Ok((
            StatusCode::BAD_REQUEST,
            Json(json!({"error":"Exicutable size is more than 20 mb"})),
        ));
    }
    let _ = sqlx::query!("Select * from tasks where id=$1", body.id)
        .fetch_one(&app_state.db_pool)
        .await
        .map_err(|e| AppError(anyhow::Error::new(e)))?;

    let expires_in = Duration::from_secs(60 * 30);

    let presigning_config =
        PresigningConfig::expires_in(expires_in).map_err(|e| AppError(anyhow::Error::new(e)))?;
    let client = app_state.config.s3.create_s3_client().await;
    let config = Arc::clone(&app_state.config);
    let presigned_url = client
        .put_object()
        .acl(ObjectCannedAcl::Private)
        .bucket(&config.s3.bucket)
        .key(body.id.to_string())
        .content_type("application/x-executable")
        .content_length(body.executable_size)
        .presigned(presigning_config)
        .await
        .map_err(|e| AppError(anyhow::Error::new(e)))?;
    let response = SignUrlResponse {
        presigned_url: presigned_url.uri().to_string(),
    };
    Ok((StatusCode::OK, Json(json!(response))))
}
