use aws_config::{BehaviorVersion, Region};

use aws_sdk_sqs::{config::Credentials, Client};
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct SQS {
    #[serde(alias = "REGION")]
    pub region: String,
    #[serde(alias = "QUEUE")]
    pub queue: String,
    #[serde(alias = "ACCESS_KEY")]
    pub access_key: String,
    #[serde(alias = "SECRET_KEY")]
    pub secret_key: String,
    #[serde(alias = "QUEUE_URL")]
    pub queue_url: String,
}
impl SQS {
    pub async fn create_client(&self) -> Client {
        let region = Region::new(self.region.clone());
        let credentials = Credentials::new(
            &self.access_key,
            &self.secret_key,
            None,
            None,
            "sqs_task_queue",
        );
        let s3_config = aws_config::defaults(BehaviorVersion::v2024_03_28())
            .region(region)
            .credentials_provider(credentials)
            .load()
            .await;
        Client::new(&s3_config)
    }
}
pub struct SQSMessage {
    pub id: String,
}
