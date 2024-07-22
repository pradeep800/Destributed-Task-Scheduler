use std::{collections::HashMap, time::SystemTime};

use chrono::Utc;
use pub_api::routes::tasks::Task;

use crate::test_helper::spawn;

#[tokio::test]
async fn check_status_of_task() {
    let app = spawn().await;
    let client = reqwest::Client::new();
    // insert a task in database
    let second = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + 30;
    let schedule_at_in_second = (second + 100) as i32;
    let retry = 3;
    let res = sqlx::query!(
        "INSERT INTO Tasks (schedule_at_in_second, status, process_time_in_second, retry, created_at) 
         VALUES ($1,$2,$3,$4,$5) RETURNING id",
        schedule_at_in_second,
        "ADDED",
        2*60*60,
        retry,
        Utc::now()
    )
    .fetch_one(&app.db_pool)
    .await
    .unwrap();

    // check status of that task
    let mut body = HashMap::new();
    body.insert("id", res.id);
    let tasks = client
        .post(format!("{}/task/status", app.address))
        .json(&body)
        .send()
        .await
        .unwrap()
        .json::<Task>()
        .await
        .unwrap();
    assert_eq!(tasks.status, "ADDED");
    assert_eq!(tasks.schedule_at_in_second, schedule_at_in_second);
    assert_eq!(tasks.retry, retry);
}
