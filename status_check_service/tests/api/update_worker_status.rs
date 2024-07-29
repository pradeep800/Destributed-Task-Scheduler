use tasks::TasksDb;

use crate::test_helpers::{generate_random_processing_task, spawn};

#[tokio::test]
pub async fn worker_sending_success_status() {
    let app = spawn().await;
    let pool = app.config.tasks_db.get_pool().await;
    let task_db = TasksDb::new(&pool);
    task_db
        .create_task(&generate_random_processing_task())
        .await
        .unwrap();
    let body = serde_json::json!({"status":"SUCCESS"});
    let client = reqwest::Client::new();
    let res = client
        .post(format!("{}/worker/update-status", app.address))
        .json(&body)
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
    let pool = app.config.tasks_db.get_pool().await;
    let task_db = TasksDb::new(&pool);
    task_db
        .create_task(&generate_random_processing_task())
        .await
        .unwrap();
    let body =
        serde_json::json!({"status":"FAILED","failed_reason":"Can't completed task in 20 minute"});
    let client = reqwest::Client::new();
    let res = client
        .post(format!("{}/worker/update-status", app.address))
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let task = task_db.find_task_by_id(1).await.unwrap();
    assert_eq!(task.current_retry, 3);
    assert_eq!(task.failed_ats.len(), 3);
}
