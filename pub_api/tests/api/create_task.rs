use chrono::{Duration, Utc};
use serde_json::json;

use crate::test_helper::spawn;
#[derive(serde::Serialize, serde::Deserialize)]
struct SucessResponse {
    id: i32,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct ErrorResponse {
    error: String,
}

#[tokio::test]
async fn sucessfully_create_a_task() {
    let app = spawn().await;
    let client = reqwest::Client::new();
    let create_task_body = json!({
        "retry": 3,
        "schedule_at": Utc::now() + Duration::minutes(1),
    });

    let res = client
        .post(format!("{}/task/create", app.address))
        .json(&create_task_body)
        .send()
        .await
        .expect("Request failed posting request to creating task");

    assert_eq!(200, res.status());
    let task_res = res.json::<SucessResponse>().await.unwrap();

    let task_db = sqlx::query!("select * from tasks")
        .fetch_all(&app.db_pool)
        .await
        .unwrap();
    assert_eq!(task_db.len(), 1);
    assert_eq!(task_db[0].id, task_res.id);
}
#[tokio::test]
async fn more_retry_than_3() {
    let app = spawn().await;
    let client = reqwest::Client::new();
    let body = json!({
        "retry": 4,
        "schedule_at": Utc::now() + Duration::minutes(1),
    });
    let res = client
        .post(format!("{}/task/create", app.address))
        .json(&body)
        .send()
        .await
        .expect("Request failed posting request to creating task");

    assert_eq!(400, res.status());
    let res_json = res.json::<ErrorResponse>().await.unwrap();
    assert_eq!(res_json.error, "tasks retry should be atleast 0 and max 3");
}

#[tokio::test]
async fn less_retry_than_0() {
    let app = spawn().await;
    let client = reqwest::Client::new();
    let body = json!({
        "retry": -1,
        "schedule_at": Utc::now() + Duration::minutes(1),
    });
    let res = client
        .post(format!("{}/task/create", app.address))
        .json(&body)
        .send()
        .await
        .expect("Request failed posting request to creating task");

    assert_eq!(400, res.status());
    let res_json = res.json::<ErrorResponse>().await.unwrap();
    assert_eq!(res_json.error, "tasks retry should be atleast 0 and max 3");
}
#[tokio::test]
async fn wrong_schedule_time() {
    let app = spawn().await;
    let client = reqwest::Client::new();
    let body = json!({
        "retry": 1,
        "schedule_at": Utc::now()-Duration::minutes(1),
    });
    let res = client
        .post(format!("{}/task/create", app.address))
        .json(&body)
        .send()
        .await
        .expect("Request failed posting request to creating task");
    assert_eq!(400, res.status());
    let res_json = res.json::<ErrorResponse>().await.unwrap();
    assert_eq!(
        res_json.error,
        "schedule_at_in_second should be alteast greater than now (in server)"
    );
}
