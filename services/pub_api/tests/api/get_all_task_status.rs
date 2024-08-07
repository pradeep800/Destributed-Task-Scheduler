use crate::test_helper::spawn;
use chrono::Utc;
use pub_api::routes::GetAllTaskStatusResponse;
use tasks::{helper_fn::TaskFaker, TasksDb};

#[tokio::test]
async fn get_all_task() {
    let app = spawn().await;
    let task_db = TasksDb::new(&app.db_pool);
    for i in 0..5 {
        let mut task = TaskFaker::generate_random_processing_task();
        if i == 0 {
            task.successful_at = Some(Utc::now());
        }
        task_db.create_task(&task).await.unwrap();
    }
    let url = format!("{}/task/all/status", app.address);
    let client = reqwest::Client::new();
    let all_task = client
        .get(url)
        .send()
        .await
        .unwrap()
        .json::<Vec<GetAllTaskStatusResponse>>()
        .await
        .unwrap();
    let mut i = 0;
    for task in all_task {
        if i == 0 {
            i += 1;
            assert_eq!(task.status, "SUCCESS")
        } else {
            assert_eq!(task.status, "PROCESSING")
        }
    }
}
