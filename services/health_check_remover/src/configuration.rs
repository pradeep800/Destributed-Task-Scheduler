use std::path::PathBuf;

use common::database::Database;
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize)]
pub struct Config {
    pub health_check: Database,
}
pub fn get_configuration() -> Config {
    let base_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let configuration_path = base_path.join("env.yaml");

    let configuration_directory = configuration_path
        .to_str()
        .expect("Failed to convert path to string");

    let settings = config::Config::builder()
        .add_source(
            config::File::new(configuration_directory, config::FileFormat::Yaml).required(false),
        )
        .add_source(
            config::Environment::with_prefix("APP")
                .prefix_separator("__")
                .separator("__"),
        )
        .build()
        .expect("Failed to build configuration");

    settings
        .try_deserialize::<Config>()
        .expect("Failed to deserialize configuration")
}
