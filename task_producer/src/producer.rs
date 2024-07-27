use aws_sdk_sqs::Client;
use std::{thread::sleep, time::Duration};

use crate::configuration::Config;

pub async fn producer(config: Config) {
    let pool = config.database.get_pool().await;
    loop {
        let mut transaction = pool.begin().await.unwrap();
        let tasks = sqlx::query!(
            "SELECT id
             FROM Tasks
             WHERE schedule_at <= now() + INTERVAL '30 seconds'
             AND is_producible= true
             AND file_uploaded = true
             ORDER BY schedule_at
             LIMIT 20
             FOR UPDATE SKIP LOCKED"
        )
        .fetch_all(&mut *transaction)
        .await
        .unwrap();
        println!("fetched all");
        let client = config.sqs.create_client().await;

        let mut task_ids: Vec<i32> = tasks.iter().map(|task| task.id).collect();

        for id in &task_ids {
            send_sqs(&client, id.to_string(), &config.sqs.queue_url).await;
        }
        println!("putted them into queue");
        sqlx::query!(
            "UPDATE Tasks
                SET is_producible = false
                WHERE id = ANY($1)",
            &task_ids
        )
        .execute(&mut *transaction)
        .await
        .unwrap();
        println!("updated is_producable to true");
        let _ = transaction.commit().await;
        sleep(Duration::from_secs(1));
    }
}
pub async fn send_sqs(client: &Client, message: String, sqs_url: &str) {
    let _rsp = client
        .send_message()
        .message_body(&message)
        .queue_url(sqs_url)
        .send()
        .await
        .unwrap();
}
