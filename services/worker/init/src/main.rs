use aws_sdk_s3::Client as S3Client;
use aws_sdk_sqs::Client as SQSClient;
use common::jwt::Jwt;
use common::tracing::{get_subscriber, init_subscriber};
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;
use tracing::{error, info, info_span};
use worker_init::configuration::get_configuration;

use health_checks::HealthCheckDb;
use std::path::Path;
use tokio::fs::File;

use std::time::Duration;
use tasks::TasksDb;
use tokio::time::sleep;
#[derive(Deserialize, Serialize, Debug)]
struct MessageData {
    pub task_id: i32,
    pub tracing_id: String,
}

#[tokio::main]
async fn main() {
    let subscriber = get_subscriber(
        "worker_init".to_string(),
        "info".to_string(),
        std::io::stdout,
    );
    init_subscriber(subscriber);
    let config = get_configuration();
    let task_db_pool = config.tasks.get_pool().await;
    let tasks_db = TasksDb::new(&task_db_pool);
    let s3_client = config.s3.create_s3_client().await;

    let health_db_pool = config.health_check.get_pool().await;
    let health_db = HealthCheckDb::new(&health_db_pool);

    let sqs_client = config.sqs.create_client().await;
    let mut message: Option<MessageData> = None;
    let mut receipt_handle: Option<String> = None;
    let mut i = 1;
    while message.is_none() {
        match receive(&sqs_client, &config.sqs.queue_url).await {
            Ok(ReceiveReturn {
                receipt_handle: rh,
                data,
            }) => {
                println!("{:?} {:?}", data, rh);
                if data.is_some() {
                    message = Some(data.unwrap());
                    receipt_handle = Some(rh.unwrap());
                    break;
                }
            }
            Err(e) => {
                error!("Error receiving message: {}", e);
            }
        }
        info!("pooling for task {} th time", i);
        i += 1;
        sleep(Duration::from_secs(1)).await;
    }
    let message = message.unwrap();
    let span = info_span!("init worker", tracing_id = message.tracing_id);

    let _ = span.enter();
    let receipt_handle = receipt_handle.unwrap();
    download_file_and_put_volume(&s3_client, &config.s3.bucket, &message.task_id.to_string())
        .await
        .unwrap();
    info!("created file worker.txt");
    let mut f = File::create(Path::new("/shared/worker.txt")).await.unwrap();
    info!("getting host name");
    let hostname = std::env::var("HOSTNAME").unwrap();
    let jwt_client = Jwt::new(config.jwt_secret);
    info!("encoding jwt");
    let jwt = jwt_client
        .encode(&message.tracing_id, message.task_id, &hostname)
        .unwrap()
        + "\n";
    info!("writing jwt");
    let _ = f.write_all(jwt.as_bytes()).await;

    info!("writing tracing_id");
    let _ = f.write_all(message.tracing_id.as_bytes()).await;
    drop(f);
    info!("Releasedthe created file");
    tasks_db
        .update_picked_at_by_workers(message.task_id)
        .await
        .unwrap();
    info!("set picked at by worker in db");
    sqs_client
        .delete_message()
        .queue_url(config.sqs.queue_url)
        .receipt_handle(receipt_handle)
        .send()
        .await
        .unwrap();
    info!("delted task from queue");
    health_db
        .cu_health_check_entries(message.task_id, &hostname)
        .await
        .unwrap();
    info!("created health entry");
    info!("completed");
}

pub async fn download_file_and_put_volume(
    client: &S3Client,
    bucket_name: &str,
    object_key: &str,
) -> Result<(), aws_sdk_s3::Error> {
    let get_object_output = client
        .get_object()
        .bucket(bucket_name)
        .key(object_key)
        .send()
        .await?;
    let mut file = File::create(Path::new("/shared/task")).await.unwrap();

    let stream = get_object_output.body;
    let data = stream.collect().await.unwrap();
    let _ = file.write_all(&data.into_bytes()).await;

    Ok(())
}
struct ReceiveReturn {
    receipt_handle: Option<String>,
    data: Option<MessageData>,
}

async fn receive(client: &SQSClient, queue_url: &str) -> Result<ReceiveReturn, aws_sdk_sqs::Error> {
    let rcv_message_output = client
        .receive_message()
        .queue_url(queue_url)
        .max_number_of_messages(1)
        .send()
        .await?;

    let (receipt_handle, data) = if let Some(messages) = rcv_message_output.messages {
        if let Some(message) = messages.first() {
            let receipt_handle = message.receipt_handle.clone();
            let data = message
                .body
                .as_ref()
                .and_then(|body| serde_json::from_str::<MessageData>(&body).ok());
            (receipt_handle, data)
        } else {
            (None, None)
        }
    } else {
        (None, None)
    };

    Ok(ReceiveReturn {
        receipt_handle,
        data,
    })
}
