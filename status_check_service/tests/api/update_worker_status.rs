use chrono::{Duration, Utc};
use tasks::{Task, TasksDb};

use crate::test_helpers::{generate_random_processing_task, spawn};

#[tokio::test]
pub async fn worker_sending_success_status() {
    let app = spawn().await;
    let task_db = TasksDb::new(app.config.tasks_db.get_pool().await);
    task_db.create_task(&generate_random_processing_task());
    let body = serde_json::json!({"status":"SUCCESS"});
    let client = reqwest::Client::new();
    let res = client
        .post(format!("{}/worker/update-status", app.address))
        .body(body)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let task = task_db.find_task_by_id(1).await.unwrap();
    assert!(task.successful_at.is_some());
}
#[tokio::test]
pub async fn worker_sending_failed_status() {
    let app = spawn().await;
    let task_db = TasksDb::new(app.config.tasks_db.get_pool().await);
    task_db.create_task(&generate_random_processing_task());
    let body = serde_json::json!({"status":"FAILED"});
    let client = reqwest::Client::new();
    let res = client
        .post(format!("{}/worker/update-status", app.address))
        .body(body)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let task = task_db.find_task_by_id(1).await.unwrap();
    assert!(task.current_retry, 3);
    assert_eq!(task.failed_ats.len(), 3);
}
