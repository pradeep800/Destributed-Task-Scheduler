use common::database::Database;
use config::{File, FileFormat};
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Config {
    pub tasks_db: Database,
    pub health_db: Database,
    #[serde(alias = "JWT_SECRET")]
    pub jwt_secret: String,
}

pub fn get_configuration() -> Config {
    config::Config::builder()
        .add_source(File::new("env.yaml", FileFormat::Yaml))
        .build()
        .unwrap()
        .try_deserialize()
        .unwrap()
}
