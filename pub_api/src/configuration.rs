use common::database::Database;
use config::{File, FileFormat};

pub fn get_configuration() -> Database {
    config::Config::builder()
        .add_source(File::new("env.yml", FileFormat::Yaml))
        .build()
        .unwrap()
        .try_deserialize()
        .unwrap()
}

