use aws_sdk_s3::presigning::PresigningConfig;
use aws_sdk_sqs::Client as SQSClient;
use common::jwt::Jwt;
use common::tracing::{get_subscriber, init_subscriber};
use serde::{Deserialize, Serialize};
use tracing::{error, info, info_span};
use worker_spinner::configuration::get_configuration;
use worker_spinner::kube::create_job;

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
    let subscriber = get_subscriber("worker".to_string(), "info".to_string(), std::io::stdout);
    init_subscriber(subscriber);

    let config = get_configuration();
    let task_db_pool = config.tasks.get_pool().await;
    let tasks_db = TasksDb::new(&task_db_pool);
    let s3_client = config.s3.create_s3_client().await;
    let sqs_client = config.sqs.create_client().await;

    loop {
        let mut message: Option<MessageData> = None;
        let mut receipt_handle: Option<String> = None;
        let mut i = 1;
        while message.is_none() {
            match receive(&sqs_client, &config.sqs.queue_url).await {
                Ok(ReceiveReturn {
                    receipt_handle: rh,
                    data,
                }) => {
                    info!("{:?} {:?}", data, rh);
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
        let _guard = span.enter();
        let receipt_handle = receipt_handle.unwrap();
        let host_id = uuid::Uuid::new_v4().to_string();
        let jwt_client = Jwt::new(config.jwt_secret.clone());
        let jwt = jwt_client
            .encode(&message.tracing_id, message.task_id, &host_id)
            .unwrap()
            + "\n";
        let expire_in = Duration::from_secs(60 * 21);
        let presigning_config = PresigningConfig::expires_in(expire_in).unwrap();
        let signed_url = s3_client
            .get_object()
            .bucket(&config.s3.bucket)
            .key(message.task_id.to_string())
            .presigned(presigning_config)
            .await
            .unwrap();

        create_job(
            signed_url.uri().to_string(),
            jwt,
            message.tracing_id,
            &host_id,
        )
        .await
        .unwrap();

        info!("job is created");
        tasks_db
            .update_picked_at_by_workers(message.task_id)
            .await
            .unwrap();
        info!("set picked at by worker in db");
        match sqs_client
            .delete_message()
            .queue_url(config.sqs.queue_url.clone())
            .receipt_handle(receipt_handle)
            .send()
            .await
        {
            Ok(_) => info!("Message deleted successfully"),
            Err(e) => {
                error!("Failed to delete message: {:?}", e);
                panic!("didn't able to delete message");
            }
        }
        info!("completed");
    }
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
