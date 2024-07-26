use chrono::{Duration, Utc};
use pub_api::routes::{SignUrlBody, SignUrlResponse};
use serde_json::json;

use crate::test_helper::{spawn, Task};
#[derive(serde::Serialize, serde::Deserialize)]
struct Error {
    error: String,
}
#[tokio::test]
async fn create_over_20mb_presigned_url() {
    let app = spawn().await;
    let client = reqwest::Client::new();
    let body = json!(SignUrlBody {
        id: 1,
        executable_size: 20 * 1024 * 1024 + 1
    });
    let res = client
        .post(format!("{}/signurl/create", app.address))
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
    let json = res.json::<Error>().await.unwrap();
    assert_eq!(json.error, "Exicutable size is more than 20 mb")
}

#[tokio::test]
async fn successfully_created_presigned_url() {
    let app = spawn().await;
    let new_task = Task {
        id: 1,
        schedule_at: Utc::now() + Duration::minutes(1),
        picked_at_by_workers: Vec::new(),
        picked_at_by_producers: Vec::new(),
        successful_at: None,
        failed_ats: Vec::new(),
        failed_reasons: Vec::new(),
        total_retry: 3,
        current_retry: 0,
        file_uploaded: true,
        is_producible: true,
        tracing_id: uuid::Uuid::new_v4().to_string(),
    };
    let _inserted_task = new_task.create_task_in_db(&app.db_pool).await;
    let client = reqwest::Client::new();
    let body = json!(SignUrlBody {
        id: 1,
        executable_size: 20 * 1024 * 1024
    });
    let res = client
        .post(format!("{}/signurl/create", app.address))
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let _json = res.json::<SignUrlResponse>().await.unwrap();
}
