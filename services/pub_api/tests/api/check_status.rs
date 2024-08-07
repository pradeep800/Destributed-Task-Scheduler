use crate::test_helper::{spawn, Task};
use chrono::{Duration, Utc};
use pub_api::routes::TaskStatusResponse;
use std::collections::HashMap;

#[tokio::test]
async fn check_status_of_task() {
    let app = spawn().await;
    let client = reqwest::Client::new();
    // insert a task in database
    // check status of that task
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

    let inserted_task = new_task.create_task_in_db(&app.db_pool).await;
    let mut body = HashMap::new();
    body.insert("id", inserted_task.id);
    let tasks = client
        .post(format!("{}/task/status", app.address))
        .json(&body)
        .send()
        .await
        .unwrap()
        .json::<TaskStatusResponse>()
        .await
        .unwrap();
    assert_eq!(tasks.status, "PROCESSING");
    assert_eq!(tasks.total_retry, new_task.total_retry);
    assert_eq!(tasks.currently_retrying_at, new_task.current_retry);
}
