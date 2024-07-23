#[derive(serde::Serialize, serde::Deserialize)]
pub struct Kafka {
    pub host: String,
}
pub fn get_bootstrap_server() -> String {}
