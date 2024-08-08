use crate::configuration::Config;
use aws_sdk_sqs::Client;
use serde_json::json;
use tokio::time::{sleep, Duration};
use tracing::{error, info, trace_span};

pub async fn producer(config: &Config) {
    let pool = config.tasks.get_pool().await;
    let mut sqs_retry = 0;
    let span = trace_span!("trace_span");
    let _ = span.enter();

    'outer: loop {
        match process_batch(config, &pool).await {
            Ok(()) => {
                sqs_retry = 0;
                sleep(Duration::from_secs(1)).await;
            }
            Err(ProcessError::SqsError) => {
                sqs_retry += 1;
                if sqs_retry == 3 {
                    break 'outer;
                }
            }
            Err(ProcessError::DbError) => {
                sleep(Duration::from_secs(2)).await;
            }
        }
    }
}

enum ProcessError {
    SqsError,
    DbError,
}
#[derive(serde::Serialize, serde::Deserialize)]
pub struct SqsBody {
    pub task_id: i32,
    pub tracing_id: String,
}

async fn process_batch(config: &Config, pool: &sqlx::PgPool) -> Result<(), ProcessError> {
    let mut transaction = pool.begin().await.map_err(|_| ProcessError::DbError)?;

    let tasks = sqlx::query!(
        "SELECT id, tracing_id
         FROM Tasks
         WHERE schedule_at < now() + INTERVAL '30 seconds'
         AND is_producible = true
         AND file_uploaded = true
         ORDER BY id
         LIMIT 20
         FOR UPDATE SKIP LOCKED"
    )
    .fetch_all(&mut *transaction)
    .await
    .map_err(|_| ProcessError::DbError)?;
    let client = config.sqs.create_client().await;
    for task in &tasks {
        info!("Producer produce tasks with tracing id {}", task.tracing_id);
        if let Err(err) = send_sqs(
            &client,
            json!(SqsBody {
                tracing_id: task.tracing_id.clone(),
                task_id: task.id
            })
            .to_string(),
            &config.sqs.queue_url,
        )
        .await
        {
            error!("{:?}", err);
            transaction
                .rollback()
                .await
                .map_err(|_| ProcessError::DbError)?;
            return Err(ProcessError::SqsError);
        }
    }

    let task_ids: Vec<i32> = tasks.iter().map(|task| task.id).collect();
    sqlx::query!(
        "UPDATE Tasks
        SET is_producible = false
        WHERE id = ANY($1)",
        &task_ids
    )
    .execute(&mut *transaction)
    .await
    .map_err(|_| ProcessError::DbError)?;

    transaction
        .commit()
        .await
        .map_err(|_| ProcessError::DbError)?;

    Ok(())
}

pub async fn send_sqs(
    client: &Client,
    message: String,
    sqs_url: &str,
) -> Result<(), aws_sdk_sqs::Error> {
    let _response = client
        .send_message()
        .message_body(message)
        .queue_url(sqs_url)
        .send()
        .await?;

    Ok(())
}
