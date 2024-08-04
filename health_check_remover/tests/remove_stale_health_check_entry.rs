use common::helper::spawn;
use health_check_remover::process::process;
use health_checks::{faker::HealthCheckEntryFaker, HealthCheckDb};
mod common;
#[tokio::test]
async fn remove_stale_entries() {
    let app = spawn().await;
    let health_db_pool = app.database.get_pool().await;
    let health_db = HealthCheckDb::new(&health_db_pool);
    for i in 0..5 {
        let mut new_health_entry = HealthCheckEntryFaker::twenty_minute_smaller_health_checks(i);
        new_health_entry.worker_finished = true;
        health_db.create(&new_health_entry).await.unwrap();
    }

    process(&health_db_pool).await;
    let all_health_entries = health_db.select_all().await.unwrap();
    assert_eq!(all_health_entries.len(), 0);
}
