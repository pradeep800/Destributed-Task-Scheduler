use common::{database::Database, s3::S3};
use config::{File, FileFormat};
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Config {
    pub s3: S3,
    pub database: Database,
}

pub fn get_configuration() -> Config {
    config::Config::builder()
        .add_source(File::new("env.yaml", FileFormat::Yaml))
        .build()
        .unwrap()
        .try_deserialize()
        .unwrap()
}
