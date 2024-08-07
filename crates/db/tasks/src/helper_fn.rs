use chrono::{Duration, Utc};

use crate::Task;

pub struct TaskFaker;
impl TaskFaker {
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
            schedule_at: Utc::now() - Duration::seconds(1),
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
