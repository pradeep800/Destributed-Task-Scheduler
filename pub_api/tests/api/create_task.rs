use std::collections::HashMap;
use std::time::SystemTime;

use crate::test_helper::spawn;
#[derive(serde::Serialize, serde::Deserialize)]
struct SucessResponse {
    id: i32,
    status: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct ErrorResponse {
    error: String,
}
#[tokio::test]
async fn sucessfully_create_a_task() {
    let app = spawn().await;
    let client = reqwest::Client::new();
    let mut body = HashMap::new();
    body.insert("retry", 3);
    let second = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + 30;
    body.insert("schedule_at_in_second", second);
    body.insert("process_time_in_second", 2 * 60 * 60);
    let res = client
        .post(format!("{}/task/create", app.address))
        .json(&body)
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
async fn more_retry_than() {
    let app = spawn().await;
    let client = reqwest::Client::new();
    let mut body = HashMap::new();
    body.insert("retry", 4);
    body.insert("process_time_in_second", 2 * 60 * 60);
    let second = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + 30;
    body.insert("schedule_at_in_second", second);
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
async fn less_retry_than() {
    let app = spawn().await;
    let client = reqwest::Client::new();
    let mut body: HashMap<&str, i32> = HashMap::new();
    body.insert("retry", -1);
    body.insert("process_time_in_second", 2 * 60 * 60);

    let second = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + 30;
    body.insert("schedule_at_in_second", second as i32);
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
    let mut body = HashMap::new();
    body.insert("retry", 2);
    body.insert("process_time_in_second", 2 * 60 * 60);
    let second = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        - 30;
    body.insert("schedule_at_in_second", second as i32);
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

#[tokio::test]
async fn process_time_in_second_more_than_20_minute() {
    let app = spawn().await;
    let client = reqwest::Client::new();
    let mut body = HashMap::new();
    body.insert("retry", 0);

    body.insert("process_time_in_second", 21 * 60 * 60);
    let second = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + 30;
    body.insert("schedule_at_in_second", second as i32);

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
        "process_time_in_second should be less than equal to 20 minute (in second)"
    );
}
