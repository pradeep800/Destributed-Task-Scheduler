use axum::{
    extract::{Extension, Json, State},
    response::IntoResponse,
};
use health_checks::HealthCheckDb;
use std::sync::Arc;
use tasks::TasksDb;
use tracing::{info, info_span};

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
    let mut task_transaction = state
        .task_pool
        .begin()
        .await
        .map_err(|e| AppError::InternalServerError(anyhow::Error::new(e)))?;
    let mut health_check_transaction = state
        .health_check_pool
        .begin()
        .await
        .map_err(|e| AppError::InternalServerError(anyhow::Error::new(e)))?;
    //lets get lock for  Task which we currently processing
    let current_task = TasksDb::get_lock_with_id(&mut task_transaction, claims.task_id)
        .await
        .map_err(|e| AppError::InternalServerError(anyhow::Error::new(e)))?;
    //our  worker is finished  (successful/unsuccessful)
    let span = info_span!("tracing_id ={}", current_task.tracing_id);
    let _guard = span.enter();

    HealthCheckDb::worker_finished(
        &mut health_check_transaction,
        claims.task_id,
        claims.pod_name.as_str(),
    )
    .await
    .map_err(|e| AppError::InternalServerError(anyhow::Error::new(e)))?;
    info!("updated worker as completed");
    if body.status == "SUCCESS" {
        info!("added successful entry to tasks");
        TasksDb::update_successful_task_with_id(&mut task_transaction, claims.task_id)
            .await
            .map_err(|x| AppError::InternalServerError(anyhow::Error::new(x)))?;
    }
    // if worker send us failed
    else if body.status == "FAILED" {
        info!("adding failing entry to tasks");
        // we will check if total_retry > current_retry do these operation
        // current_retry++
        // failed_at=now
        // is_producible=true
        // failed_reason=reason
        if current_task.total_retry > current_task.current_retry {
            info!("total_retry > current_retry");
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
            info!("total_retry = current_retry");
            TasksDb::add_failed_with_trans(
                &mut task_transaction,
                claims.task_id,
                &body.failed_reason.unwrap_or("".to_string()),
            )
            .await
            .map_err(|x| AppError::InternalServerError(anyhow::Error::new(x)))?;
        }
    }
    //  Unknown status
    else {
        // If Unknown status rollback the transaction
        info!("find unknown status");
        task_transaction
            .rollback()
            .await
            .map_err(|e| AppError::InternalServerError(anyhow::Error::new(e)))?;
        health_check_transaction
            .rollback()
            .await
            .map_err(|e| AppError::InternalServerError(anyhow::Error::new(e)))?;

        return Err(AppError::InternalServerError(anyhow::anyhow!(
            "Unknown type of status"
        )));
    }
    info!("committed everything");
    task_transaction
        .commit()
        .await
        .map_err(|e| AppError::InternalServerError(anyhow::Error::new(e)))?;
    health_check_transaction
        .commit()
        .await
        .map_err(|e| AppError::InternalServerError(anyhow::Error::new(e)))?;
    Ok(())
}
