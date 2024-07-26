use std::sync::Arc;

use axum::{extract::State, response::IntoResponse, Json};
use reqwest::StatusCode;
use serde_json::json;

use crate::state::AppState;

use super::AppError;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct UploadStatusBody {
    pub id: i32,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct UploadStatusResponse {
    pub status: String, //UPLOADED PENDING
}

pub async fn upload_status(
    State(state): State<Arc<AppState>>,
    Json(body): Json<UploadStatusBody>,
) -> Result<impl IntoResponse, AppError> {
    let s3_client = state.config.s3.create_s3_client().await;
    let status = match s3_client
        .head_object()
        .key(body.id.to_string())
        .bucket(state.config.s3.bucket.clone())
        .send()
        .await
    {
        Ok(_) => {
            match sqlx::query!(
                r#"
                UPDATE Tasks
                SET file_uploaded = true
                WHERE id = $1
                "#,
                body.id
            )
            .execute(&state.db_pool)
            .await
            {
                Ok(_) => "UPLOADED",
                Err(e) => return Err(AppError(anyhow::Error::new(e))),
            }
        }
        Err(_) => "PENDING",
    };

    Ok((
        StatusCode::OK,
        Json(json!(UploadStatusResponse {
            status: status.to_string()
        })),
    ))
}
