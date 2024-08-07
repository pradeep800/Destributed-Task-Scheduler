use chrono::{Duration, Utc};

use crate::HealthCheck;
pub struct HealthCheckEntryFaker;
impl HealthCheckEntryFaker {
    pub fn unintended_behaviour_worker(id: i32) -> HealthCheck {
        HealthCheck {
            task_id: id,
            last_time_health_check: Utc::now() - Duration::seconds(30),
            worker_finished: false,
            pod_name: "abc_123".to_string(),
        }
    }
    pub fn twenty_minute_smaller_health_checks(id: i32) -> HealthCheck {
        HealthCheck {
            task_id: id,
            last_time_health_check: Utc::now() - Duration::minutes(21),
            worker_finished: false,
            pod_name: "abc_123".to_string(),
        }
    }
}
