use common::{database::Database, kafka_producer::Kafka};

use config::{File, FileFormat};
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Config {
    pub database: Database,
    pub kafka: Kafka,
}

pub fn get_configuration() -> Config {
    config::Config::builder()
        .add_source(File::new("env.yml", FileFormat::Yaml))
        .build()
        .unwrap()
        .try_deserialize()
        .unwrap()
}
