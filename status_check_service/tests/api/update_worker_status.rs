use common::jwt::Jwt;
use health_checks::HealthCheckDb;
use tasks::TasksDb;

use crate::test_helpers::{generate_random_processing_task, spawn};
use axum::http::{HeaderMap, HeaderValue};

#[tokio::test]
pub async fn worker_sending_success_status() {
    let app = spawn().await;
    let pool = app.config.tasks_db.get_pool().await;
    let task_db = TasksDb::new(&pool);

    let health_pool = app.config.health_db.get_pool().await;
    let health_db = HealthCheckDb::new(&health_pool);
    let new_task = generate_random_processing_task();
    task_db.create_task(&new_task).await.unwrap();
    let _ = health_db
        .cu_health_check_entries(new_task.id, "pod_123")
        .await;
    let body = serde_json::json!({"status":"SUCCESS"});
    let jwt = Jwt::new(app.config.jwt_secret);
    let jwt_token = jwt
        .encode(&new_task.tracing_id, new_task.id, "pod_123")
        .unwrap();

    let mut headers = HeaderMap::new();

    headers.insert(
        reqwest::header::AUTHORIZATION,
        HeaderValue::from_str(&jwt_token).unwrap(),
    );
    let client = reqwest::Client::new();

    let res = client
        .post(format!("{}/worker/update-status", app.address))
        .json(&body)
        .headers(headers)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let task = task_db.find_task_by_id(1).await.unwrap();
    assert!(task.successful_at.is_some());

    let health_entry = health_db.find_entry(1, "pod_123").await.unwrap();

    assert_eq!(health_entry.worker_finished, true);
}
#[tokio::test]
pub async fn worker_sending_failed_status() {
    let app = spawn().await;
    let pool = app.config.tasks_db.get_pool().await;
    let task_db = TasksDb::new(&pool);
    let new_task = generate_random_processing_task();
    task_db.create_task(&new_task).await.unwrap();
    let jwt = Jwt::new(app.config.jwt_secret);
    let jwt_token = jwt
        .encode(&new_task.tracing_id, new_task.id, "pod_123")
        .unwrap();

    let mut headers = HeaderMap::new();

    headers.insert(
        reqwest::header::AUTHORIZATION,
        HeaderValue::from_str(&jwt_token).unwrap(),
    );
    let body =
        serde_json::json!({"status":"FAILED","failed_reason":"Can't completed task in 20 minute"});
    let client = reqwest::Client::new();
    let res = client
        .post(format!("{}/worker/update-status", app.address))
        .json(&body)
        .headers(headers)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let task = task_db.find_task_by_id(1).await.unwrap();
    assert_eq!(task.current_retry, 3);
    assert_eq!(task.failed_ats.len(), 3);
}
#[tokio::test]
pub async fn worker_sending_random_status() {
    let app = spawn().await;
    let pool = app.config.tasks_db.get_pool().await;
    let task_db = TasksDb::new(&pool);
    let new_task = generate_random_processing_task();
    task_db.create_task(&new_task).await.unwrap();
    let jwt = Jwt::new(app.config.jwt_secret);
    let jwt_token = jwt
        .encode(&new_task.tracing_id, new_task.id, "pod_123")
        .unwrap();

    let mut headers = HeaderMap::new();

    headers.insert(
        reqwest::header::AUTHORIZATION,
        HeaderValue::from_str(&jwt_token).unwrap(),
    );
    let body =
        serde_json::json!({"status":"random","failed_reason":"Can't completed task in 20 minute"});
    let client = reqwest::Client::new();
    let res = client
        .post(format!("{}/worker/update-status", app.address))
        .json(&body)
        .headers(headers)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 500);
}
