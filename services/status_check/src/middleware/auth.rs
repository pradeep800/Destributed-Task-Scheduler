use crate::{error::AppError, startup::AppState};
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use common::jwt::Jwt;
use std::sync::Arc;
pub async fn auth(
    State(state): State<Arc<AppState>>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let auth_header = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or(AppError::Unauthorized)?;
    let jwt = Jwt::new(state.config.jwt_secret.to_string());
    let claim = jwt
        .verify(auth_header)
        .map_err(|_e| AppError::Unauthorized)?;
    req.extensions_mut().insert(claim);
    Ok(next.run(req).await)
}
