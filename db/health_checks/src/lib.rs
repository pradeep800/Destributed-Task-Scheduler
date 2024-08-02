pub mod faker;
use chrono::{DateTime, Utc};
use sqlx::{query, PgPool, Postgres, Transaction};
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
    pub async fn select_all(&self) -> Result<Vec<HealthCheck>, sqlx::Error> {
        let health_checks = sqlx::query_as!(HealthCheck, "select * from health_check_entries")
            .fetch_all(self.pool)
            .await?;
        Ok(health_checks)
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
            last_time_health_check = NOW(),
            worker_finished=false
            ",
            task_id,
            pod_name
        )
        .execute(self.pool)
        .await?;
        Ok(())
    }
    pub async fn create(&self, health_entry: &HealthCheck) -> Result<(), sqlx::Error> {
        query!(
            "INSERT INTO health_check_entries (task_id, last_time_health_check,pod_name,worker_finished)
            VALUES ($1, $2,$3,$4)
            ",
            health_entry.task_id,
           health_entry.last_time_health_check,
            health_entry.pod_name,
            health_entry.worker_finished
        )
        .execute(self.pool)
        .await?;
        Ok(())
    }
    pub async fn worker_finished(
        health_trans: &mut Transaction<'_, Postgres>,
        task_id: i32,
        pod_name: &str,
    ) -> Result<(), sqlx::Error> {
        query!(
            "UPDATE health_check_entries SET 
         worker_finished=true
         WHERE task_id=$1 AND
         pod_name=$2 AND
         worker_finished=false
         ",
            task_id,
            pod_name
        )
        .execute(&mut **health_trans)
        .await?;
        Ok(())
    }
    pub async fn get_10_dead_health_entries(
        health_trans: &mut Transaction<'_, Postgres>,
    ) -> Result<Vec<HealthCheck>, sqlx::Error> {
        let health_entries = sqlx::query_as!(
            HealthCheck,
            "
            SELECT *
            FROM health_check_entries
            WHERE last_time_health_check < NOW() - INTERVAL '20 seconds'
            AND worker_finished = false
            ORDER BY task_id, pod_name
            LIMIT 10
            FOR UPDATE SKIP LOCKED
           "
        )
        .fetch_all(&mut **health_trans)
        .await?;
        Ok(health_entries)
    }
}
