use axum::{http::StatusCode, response::IntoResponse};

#[derive(thiserror::Error, Debug)]
#[error("Internal Server Error")]
struct AppError(anyhow::Error);
impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", self.0)).into_response()
    }
}
pub async fn create_task() -> Result<impl IntoResponse, AppError> {
    Ok((StatusCode::OK, "hello"))
}
