use chrono::{DateTime, Utc};
use sqlx::{query, PgPool};

pub struct HealthCheckDb<'a> {
    pub pool: &'a PgPool,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct HealthCheck {
    pub task_id: i32,
    pub last_time_health_check: DateTime<Utc>,
    pub task_completed: bool,
}

impl<'a> HealthCheckDb<'a> {
    pub fn new(pool: &'a PgPool) -> HealthCheckDb<'a> {
        HealthCheckDb { pool }
    }

    pub async fn select_with_task_id_in_health_db(
        &self,
        id: i32,
    ) -> Result<HealthCheck, sqlx::Error> {
        let health_check_info = sqlx::query_as!(
            HealthCheck,
            "SELECT * FROM health_check_entries WHERE task_id = $1",
            id
        )
        .fetch_one(self.pool)
        .await?;
        Ok(health_check_info)
    }
    pub async fn create_update_last_time_health_check(
        &self,
        task_id: i32,
    ) -> Result<(), sqlx::Error> {
        query!(
            "INSERT INTO health_check_entries (task_id, last_time_health_check)
            VALUES ($1, NOW())
            ON CONFLICT (task_id)
            DO UPDATE SET
            last_time_health_check = NOW()",
            task_id
        )
        .execute(self.pool)
        .await?;
        Ok(())
    }
    pub async fn task_completed(&self, task_id: i32) -> Result<(), sqlx::Error> {
        query!(
            "UPDATE health_check_entries SET task_completed=true WHERE task_id=$1",
            task_id
        )
        .execute(self.pool)
        .await?;
        Ok(())
    }
}
