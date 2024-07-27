use axum::http::{HeaderMap, HeaderValue};
use common::jwt::Jwt;
use health_checks::HealthCheckDb;
/*
* Authorization: bearer <token>
* task_id
* tracing_id
*/
use tasks::TaskDb;

use crate::test_helpers::spawn;
#[tokio::test]
async fn check_heart_beat() {
    let app = spawn().await;
    let task_db = TaskDb::new(app.config.tasks_db.get_pool().await);
    let new_task = TaskDb::generate_random_processing_task();
    task_db.create_task(&new_task).await.unwrap();
    let jwt = Jwt::new(app.config.jwt_secret);
    let jwt_token = jwt.encode(&new_task.tracing_id, new_task.id).unwrap();
    let mut headers = HeaderMap::new();
    headers.insert(
        reqwest::header::AUTHORIZATION,
        HeaderValue::from_str(&jwt_token).unwrap(),
    );
    let client = reqwest::Client::new();
    let req = client
        .get(format!("{}/worker/heart-beat", app.address))
        .headers(headers)
        .send()
        .await
        .unwrap();
    assert_eq!(req.status(), 200);
    let pool = app.config.health_db.get_pool().await;
    let health_check_db = HealthCheckDb::new(&pool);
    //  check if exists or not

    let _ = health_check_db
        .select_with_task_id_in_health_db(new_task.id)
        .await
        .unwrap();
}
#[tokio::test]
async fn unauthorized_heart_beat() {
    let app = spawn().await;
    let task_db = TaskDb::new(app.config.tasks_db.get_pool().await);
    let new_task = TaskDb::generate_random_processing_task();
    task_db.create_task(&new_task).await.unwrap();

    let client = reqwest::Client::new();
    let req = client
        .get(format!("{}/worker/heart-beat", app.address))
        .send()
        .await
        .unwrap();
    assert_eq!(req.status(), 401);
}
