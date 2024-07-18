use config::{Config, File, FileFormat};

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Database {
    #[serde(alias = "DATABASE_USER")]
    pub database_user: String,

    #[serde(alias = "DATBASE_DB")]
    pub database_db: String,

    #[serde(alias = "DATBASE_PASSWORD")]
    pub database_password: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct EnvVariable {
    pub database: Database,
}
pub fn get_configuration() -> EnvVariable {
    let builder = Config::builder().add_source(File::new("env.yaml", FileFormat::Yaml));
    let config = builder.build().unwrap();
    config.try_deserialize().unwrap()
}
