use aws_sdk_s3::{
    error::SdkError,
    operation::copy_object::{CopyObjectError, CopyObjectOutput},
    Client as S3Client,
};
use aws_sdk_sqs::Client as SQSClient;
use common::jwt::Jwt;
use init::configuration::get_configuration;

use common::tracing::{get_subscriber, init_subscriber};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::time::Duration;
use tasks::TasksDb;
use tokio::time::sleep;
#[derive(Deserialize, Serialize)]
struct MessageData {
    pub task_id: i32,
    pub tracing_id: String,
}

#[tokio::main]
async fn main() {
    let subscriber = get_subscriber(
        "pub_task_scheduler_api".to_string(),
        "info".to_string(),
        std::io::stdout,
    );
    init_subscriber(subscriber);
    let config = get_configuration();
    let task_db_pool = config.database.get_pool().await;
    let tasks_db = TasksDb::new(&task_db_pool);
    let s3_client = config.s3.create_s3_client().await;

    let sqs_client = config.sqs.create_client().await;
    let mut message: Option<MessageData> = None;
    let mut receipt_handle: Option<String> = None;
    while message.is_none() {
        match receive(&sqs_client, &config.sqs.queue_url).await {
            Ok(ReceiveReturn {
                receipt_handle: rh,
                data,
            }) => {
                if data.is_some() {
                    message = Some(data.unwrap());
                    receipt_handle = Some(rh.unwrap());
                }
                break;
            }
            Err(e) => {
                println!("Error receiving message: {}", e);
            }
        }
        sleep(Duration::from_secs(1)).await;
    }
    let message = message.unwrap();
    let receipt_handle = receipt_handle.unwrap();
    copy_object(&s3_client, &config.s3.bucket, &message.task_id.to_string())
        .await
        .unwrap();

    let mut f = File::create_new("/shared/foo.txt").unwrap();
    let jwt_client = Jwt::new(config.jwt_secret);
    let hostname = std::env::var("HOST_NAME").unwrap();
    let jwt = jwt_client
        .encode(&message.tracing_id, message.task_id, &hostname)
        .unwrap()
        + "\n";

    f.write(jwt.as_bytes()).unwrap();
    f.write(message.tracing_id.as_bytes()).unwrap();
    drop(f);
    tasks_db
        .update_picked_at_by_workers(message.task_id)
        .await
        .unwrap();
    sqs_client
        .delete_message()
        .queue_url(config.sqs.queue_url)
        .receipt_handle(receipt_handle)
        .send()
        .await
        .unwrap();
}

pub async fn copy_object(
    client: &S3Client,
    bucket_name: &str,
    object_key: &str,
) -> Result<CopyObjectOutput, SdkError<CopyObjectError>> {
    let mut source_bucket_and_object: String = "".to_owned();
    source_bucket_and_object.push_str(bucket_name);
    source_bucket_and_object.push('/');
    source_bucket_and_object.push_str(object_key);

    client
        .copy_object()
        .copy_source(source_bucket_and_object)
        .bucket(bucket_name)
        .key("/shared/")
        .send()
        .await
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
