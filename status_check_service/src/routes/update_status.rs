use axum::{
    extract::{Extension, Json, State},
    response::IntoResponse,
};
use health_checks::HealthCheckDb;
use std::sync::Arc;
use tasks::TasksDb;

use crate::{error::AppError, startup::AppState};
use common::jwt::Claims;
#[derive(serde::Serialize, serde::Deserialize)]
pub struct UpdateStatusBody {
    pub status: String,
    pub failed_reason: Option<String>,
}

pub async fn update_status(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<UpdateStatusBody>,
) -> Result<impl IntoResponse, AppError> {
    let task_db = TasksDb::new(&state.task_pool);
    let health_check_db = HealthCheckDb::new(&state.health_check_pool);
    // when  worker send us success
    if body.status == "SUCCESS" {
        // update the task as success and health check  task_completed as true
        task_db
            .update_successful_task_with_id(claims.task_id)
            .await
            .map_err(|x| AppError::InternalServerError(anyhow::Error::new(x)))?;
        health_check_db
            .task_completed(claims.task_id)
            .await
            .map_err(|x| AppError::InternalServerError(anyhow::Error::new(x)))?;
    }
    // if worker send us failed
    else if body.status == "FAILED" {
        let mut task_transaction = state
            .task_pool
            .begin()
            .await
            .map_err(|e| AppError::InternalServerError(anyhow::Error::new(e)))?;
        let current_task = TasksDb::get_lock_with_id(&mut task_transaction, claims.task_id)
            .await
            .map_err(|e| AppError::InternalServerError(anyhow::Error::new(e)))?;
        // we will check if total_retry > current_retry do these operation
        // current_retry++
        // failed_at=now
        // is_producible=true
        // failed_reason=reason
        if current_task.total_retry > current_task.current_retry {
            TasksDb::increase_current_retry_add_failed_and_is_producible_true(
                &mut task_transaction,
                claims.task_id,
                &body.failed_reason.unwrap_or("".to_string()),
            )
            .await
            .map_err(|e| AppError::InternalServerError(anyhow::Error::new(e)))?;
        }
        //if total_retry == current_retry
        // we will update these things
        // failed_at=now
        // failed_reason=reason
        // is_producible=false
        // and task_completed =  true in health_check database
        else {
            TasksDb::add_failed_with_trans(
                &mut task_transaction,
                claims.task_id,
                &body.failed_reason.unwrap_or("".to_string()),
            )
            .await
            .map_err(|x| AppError::InternalServerError(anyhow::Error::new(x)))?;
            health_check_db
                .task_completed(claims.task_id)
                .await
                .map_err(|x| AppError::InternalServerError(anyhow::Error::new(x)))?;
        }
        task_transaction
            .commit()
            .await
            .map_err(|e| AppError::InternalServerError(anyhow::Error::new(e)))?;
    }
    //  Unknown status
    else {
        return Err(AppError::InternalServerError(anyhow::anyhow!(
            "Unknown type of status"
        )));
    }
    Ok(())
}
