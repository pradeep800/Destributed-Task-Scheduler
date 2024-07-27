use aws_sdk_sqs::Client;
use configuration::{get_configuration, Config};
use producer::producer;
use std::{thread::sleep, time::Duration};
pub mod configuration;
pub mod producer;
#[tokio::main]
async fn main() {
    let config = get_configuration();
    producer(config).await;
}

pub async fn receive(client: &Client, queue_url: &String) {
    let rcv_message_output = client.receive_message().send().await.unwrap();

    for message in rcv_message_output.messages.unwrap_or_default() {
        println!("Got the message: {:#?}", message);
    }
}
