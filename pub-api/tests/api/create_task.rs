use std::collections::HashMap;

use chrono::{Timelike, Utc};

use crate::test_helper::spawn;

#[tokio::test]
async fn create_a_task() {
    let app = spawn().await;
    let client = reqwest::Client::new();
    let mut body = HashMap::new();
    body.insert("retry", 3);

    body.insert("schedule_at_in_second", Utc::now().second() + 40);
    let res = client
        .post(format!("{}/task/create", app.address))
        .json(&body)
        .send()
        .await
        .expect("Request failed posting request to creating task");
    //request passed
    assert_eq!(200, res.status());
    let tasks = sqlx::query!("select * from tasks")
        .fetch_all(&app.db_pool)
        .await
        .unwrap();
    assert_eq!(tasks.len(), 1);
}
#[tokio::test]
async fn more_retry_than_400_status() {
    let app = spawn().await;
    let client = reqwest::Client::new();
    let mut body = HashMap::new();
    body.insert("retry", 4);

    body.insert("schedule_at_in_second", Utc::now().second() + 40);
    let res = client
        .post(format!("{}/task/create", app.address))
        .json(&body)
        .send()
        .await
        .expect("Request failed posting request to creating task");
    assert_eq!(400, res.status());
}
#[tokio::test]
async fn wrong_schedule_at_time_400_status() {
    let app = spawn().await;
    let client = reqwest::Client::new();
    let mut body = HashMap::new();
    body.insert("retry", 4);

    body.insert("schedule_at_in_second", Utc::now().second() - 30);
    let res = client
        .post(format!("{}/task/create", app.address))
        .json(&body)
        .send()
        .await
        .expect("Request failed posting request to creating task");
    assert_eq!(400, res.status());
}
