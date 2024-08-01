use chrono::{DateTime, Utc};
use sqlx::{query, PgPool};

pub struct HealthCheckDb<'a> {
    pub pool: &'a PgPool,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct HealthCheck {
    pub task_id: i32,
    pub last_time_health_check: DateTime<Utc>,
    pub worker_finished: bool,
    pub pod_name: String,
}

impl<'a> HealthCheckDb<'a> {
    pub fn new(pool: &'a PgPool) -> HealthCheckDb<'a> {
        HealthCheckDb { pool }
    }

    pub async fn find_entry(&self, id: i32, pod_name: &str) -> Result<HealthCheck, sqlx::Error> {
        let health_check_info = sqlx::query_as!(
            HealthCheck,
            "SELECT * FROM health_check_entries WHERE task_id = $1 AND pod_name= $2",
            id,
            pod_name
        )
        .fetch_one(self.pool)
        .await?;
        Ok(health_check_info)
    }
    pub async fn cu_health_check_entries(
        &self,
        task_id: i32,
        pod_name: &str,
    ) -> Result<(), sqlx::Error> {
        query!(
            "INSERT INTO health_check_entries (task_id, last_time_health_check,pod_name)
            VALUES ($1, NOW(),$2)
            ON CONFLICT (task_id,pod_name)
            DO UPDATE SET
            last_time_health_check = NOW()",
            task_id,
            pod_name
        )
        .execute(self.pool)
        .await?;
        Ok(())
    }
    pub async fn worker_finished(&self, task_id: i32, pod_name: &str) -> Result<(), sqlx::Error> {
        query!(
            "UPDATE health_check_entries SET worker_finished=true WHERE task_id=$1 and pod_name=$2",
            task_id,
            pod_name
        )
        .execute(self.pool)
        .await?;
        Ok(())
    }
}
