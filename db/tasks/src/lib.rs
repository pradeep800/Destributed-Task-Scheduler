use chrono::{Duration, Utc};
use sqlx::{query, PgPool, Postgres, Transaction};
pub struct TasksDb<'a> {
    pub pool: &'a PgPool,
}
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Task {
    pub id: i32,
    pub schedule_at: chrono::DateTime<Utc>,
    pub picked_at_by_producers: Vec<chrono::DateTime<Utc>>,
    pub picked_at_by_workers: Vec<chrono::DateTime<Utc>>,
    pub successful_at: Option<chrono::DateTime<Utc>>,
    pub failed_ats: Vec<chrono::DateTime<Utc>>,
    pub failed_reasons: Vec<String>,
    pub total_retry: i16,
    pub current_retry: i16,
    pub file_uploaded: bool,
    pub is_producible: bool,
    pub tracing_id: String,
}
impl<'a> TasksDb<'a> {
    pub fn new(pool: &PgPool) -> TasksDb {
        TasksDb { pool }
    }
    pub async fn find_task_by_id(&self, task_id: i32) -> Result<Task, sqlx::Error> {
        sqlx::query_as!(Task, "select * from tasks where id = $1", task_id)
            .fetch_one(self.pool)
            .await
    }
    pub async fn create_task(&self, task: &Task) -> Result<(), sqlx::Error> {
        sqlx::query_as!(
            Task,
            r#"
        INSERT INTO Tasks (
              schedule_at, picked_at_by_producers, picked_at_by_workers,
            successful_at, failed_ats, failed_reasons,
            total_retry, current_retry,
            file_uploaded, is_producible, tracing_id
        )
        VALUES (
            $1, $2, $3,
            $4, $5, $6,
            $7, $8,
            $9, $10, $11 
        )
        "#,
            task.schedule_at,
            &task.picked_at_by_producers,
            &task.picked_at_by_workers,
            task.successful_at,
            &task.failed_ats,
            &task.failed_reasons,
            task.total_retry,
            task.current_retry,
            task.file_uploaded,
            task.is_producible,
            task.tracing_id,
        )
        .execute(self.pool)
        .await?;

        Ok(())
    }

    pub async fn update_successful_task_with_id(&self, id: i32) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE tasks
        SET successful_at = NOW()
        WHERE id = $1",
            id
        )
        .execute(self.pool)
        .await?;
        Ok(())
    }
    pub async fn get_lock_with_id(
        task_trans: &mut Transaction<'_, Postgres>,
        id: i32,
    ) -> Result<Task, sqlx::Error> {
        let current_task = sqlx::query_as!(Task, "SELECT * from tasks WHERE id= $1 FOR UPDATE", id)
            .fetch_one(&mut **task_trans)
            .await?;
        Ok(current_task)
    }
    pub async fn increase_current_retry_add_failed_and_is_producible_true(
        task_trans: &mut Transaction<'_, Postgres>,
        id: i32,
        failed_reason: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE tasks SET failed_ats=array_append(failed_ats,now()),
                                  failed_reasons=array_append(failed_reasons,$1),
                                  current_retry=current_retry+1,
                                  is_producible=true
                         WHERE id=$2",
            failed_reason,
            id
        )
        .execute(&mut **task_trans)
        .await?;
        Ok(())
    }
    pub async fn add_failed_with_trans(
        task_trans: &mut Transaction<'_, Postgres>,
        id: i32,
        failed_reason: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE tasks SET failed_ats=array_append(failed_ats,now()),
                              is_producible=false,
                              failed_reasons=array_append(failed_reasons,$1)
                         WHERE id=$2",
            failed_reason,
            id
        )
        .execute(&mut **task_trans)
        .await?;
        Ok(())
    }
}
