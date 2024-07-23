use std::time::SystemTime;

use configuration::get_configuration;
pub mod configuration;
#[tokio::main]
async fn main() {
    let config = get_configuration();
    let pool = config.database.get_pool().await;
    loop {
        let mut transaction = pool.begin().await.unwrap();
        let today_time_in_second = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i32;
        let today_after_30_second = today_time_in_second + 30;
        let tasks = sqlx::query!(
            "select * from tasks where schedule_at_in_second >= $1 and schedule_at_in_second <= $2",
            today_time_in_second,
            today_after_30_second
        )
        .fetch_all(&mut *transaction)
        .await
        .unwrap();
        //now we can put these into our kafka (message broker of any kind)
    }
}
