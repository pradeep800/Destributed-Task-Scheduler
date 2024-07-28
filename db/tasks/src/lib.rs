use chrono::{Duration, Utc};
use sqlx::PgPool;
pub struct TasksDb {
    pub pool: PgPool,
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
impl TasksDb {
    pub fn new(pool: PgPool) -> TasksDb {
        TasksDb { pool }
    }
    pub async fn find_task_by_id(&self, task_id: i32) -> Result<Task, sqlx::Error> {
        sqlx::query_as!(Task, "select * from tasks where id = $1", task_id)
            .fetch_one(&self.pool)
            .await
    }
    pub async fn create_task(&self, task: &Task) -> Result<(), sqlx::Error> {
        sqlx::query_as!(
            Task,
            r#"
        INSERT INTO Tasks (
             id, schedule_at, picked_at_by_producers, picked_at_by_workers,
            successful_at, failed_ats, failed_reasons,
            total_retry, current_retry,
            file_uploaded, is_producible, tracing_id
        )
        VALUES (
            $1, $2, $3,
            $4, $5, $6,
            $7, $8,
            $9, $10, $11, $12
        )
        "#,
            task.id,
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
        .fetch_one(&self.pool)
        .await?;
        Ok(())
    }
    pub fn generate_random_processing_task() -> Task {
        let mut failed_reasons = Vec::<String>::new();

        let mut failed_ats = Vec::<chrono::DateTime<Utc>>::new();
        let mut picked_at_by_producers = Vec::<chrono::DateTime<Utc>>::new();

        let mut picked_at_by_workers = Vec::<chrono::DateTime<Utc>>::new();
        for i in 0..3 {
            failed_reasons.push(format!("Failed reason {}", i + 1));
            failed_ats.push(Utc::now());
            picked_at_by_producers.push(Utc::now());
            picked_at_by_workers.push(Utc::now());
        }
        failed_ats.pop();
        failed_reasons.pop();
        let new_task = Task {
            id: 1,
            schedule_at: Utc::now() + Duration::minutes(1),
            picked_at_by_workers,
            picked_at_by_producers,
            successful_at: None,
            failed_ats,
            failed_reasons,
            total_retry: 3,
            current_retry: 2,
            file_uploaded: true,
            is_producible: true,
            tracing_id: uuid::Uuid::new_v4().to_string(),
        };
        new_task
    }
}
