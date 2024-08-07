use aws_config::{BehaviorVersion, Region};
use aws_sdk_s3::{config::Credentials, Client};

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct S3 {
    #[serde(alias = "REGION")]
    pub region: String,
    #[serde(alias = "BUCKET")]
    pub bucket: String,
    #[serde(alias = "ACCESS_KEY")]
    pub access_key: String,
    #[serde(alias = "SECRET_KEY")]
    pub secret_key: String,
}
impl S3 {
    pub async fn create_s3_client(&self) -> Client {
        let region = Region::new(self.region.clone());
        let credentials = Credentials::new(
            &self.access_key,
            &self.secret_key,
            None,
            None,
            "s3_presigned_url",
        );
        let s3_config = aws_config::defaults(BehaviorVersion::v2024_03_28())
            .region(region)
            .credentials_provider(credentials)
            .load()
            .await;
        Client::new(&s3_config)
    }
}
