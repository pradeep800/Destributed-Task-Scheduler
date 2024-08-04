use std::time::Duration;

use health_checks::{faker::HealthCheckEntryFaker, HealthCheckDb};
use retry_and_failed_updater_service::process::process;
use tasks::{helper_fn::TaskFaker, TasksDb};
use tokio::time::sleep;

use crate::helper::spawn;

#[tokio::test]
async fn update_failed_task() {
    let service = spawn().await;
    let task_db_pool = service.config.tasks.get_pool().await;
    let task_db = TasksDb::new(&task_db_pool);
    let health_db_pool = service.config.health_check.get_pool().await;

    let health_db = HealthCheckDb::new(&health_db_pool);
    // these try cannot have retry because total = current
    for _i in 0..11 {
        let mut new_task = TaskFaker::generate_random_processing_task();
        new_task.is_producible = false;
        new_task.current_retry = new_task.total_retry;
        let current_task = task_db.create_task(&new_task).await.unwrap();
        let new_health_entry = HealthCheckEntryFaker::unintended_behaviour_worker(current_task.id);
        health_db.create(&new_health_entry).await.unwrap();
    }
    // these task  can have retry
    for _i in 0..5 {
        let mut new_task = TaskFaker::generate_random_processing_task();
        new_task.is_producible = false;
        let current_task = task_db.create_task(&new_task).await.unwrap();

        let health_entry = HealthCheckEntryFaker::unintended_behaviour_worker(current_task.id);
        health_db.create(&health_entry).await.unwrap();
    }
    let updater = process(&service.config);
    let sleep = sleep(Duration::from_secs(5));
    tokio::select! {
        _=updater=>{
        },
        _=sleep=>{
        }
    }
    let tasks = task_db.select_all().await.unwrap();
    let mut it = 0;
    tasks.iter().for_each(|x| {
        if x.is_producible == true {
            it += 1;
        }
    });
    assert_eq!(it, 5);
    let health_entries = health_db.select_all().await.unwrap();
    let mut ubw_count = 0;
    health_entries.into_iter().for_each(|x| {
        if x.worker_finished == true {
            ubw_count += 1;
        }
    });

    assert_eq!(ubw_count, 16);
}
