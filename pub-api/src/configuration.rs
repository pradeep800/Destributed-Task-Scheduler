use config::{Config, File, FileFormat};

use serde_aux::field_attributes::deserialize_number_from_string;
use sqlx::{postgres::PgPoolOptions, PgPool};
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Database {
    #[serde(alias = "DATABASE_USER")]
    pub database_user: String,
    #[serde(alias = "DATBASE_DB")]
    pub database_db: String,
    #[serde(alias = "DATBASE_PASSWORD")]
    pub database_password: String,
    #[serde(alias = "DATABASE_PORT")]
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub database_port: u16,
    #[serde(alias = "DATABASE_HOST")]
    pub database_host: String,
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
impl Database {
    pub fn get_connecting_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.database_user,
            self.database_password,
            self.database_host,
            self.database_port,
            self.database_db
        )
    }
    pub fn get_connecting_string_without_db(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}",
            self.database_user, self.database_password, self.database_host, self.database_port,
        )
    }
    pub async fn get_pool(&self) -> PgPool {
        let connecting_string = self.get_connecting_string();
        PgPoolOptions::new()
            .max_connections(5)
            .connect(&connecting_string)
            .await
            .expect("could not able to connect database")
    }
}
