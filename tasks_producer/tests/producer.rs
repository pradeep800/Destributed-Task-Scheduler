mod helper;
use helper::helper::spawn;
use task_producer::producer::producer;
use tasks::helper_fn::TaskFaker;
use tasks::TasksDb;
use tokio::time::{sleep, Duration};
#[tokio::test]
async fn testing_producer() {
    let config = spawn().await;
    let task_producer = producer(&config);
    let ten_second_sleep = sleep(Duration::from_secs(10));
    let pool = config.database.get_pool().await;
    let task_db = TasksDb::new(&pool);
    let genrated_task = TaskFaker::generate_random_processing_task();
    for _i in 0..22 {
        let _ = task_db.create_task(&genrated_task).await;
    }
    tokio::select! {
        _ = task_producer => {
            println!("task_producer completed");
        }
        _ = ten_second_sleep => {
            println!("10 seconds elapsed");
        }
    }
}
