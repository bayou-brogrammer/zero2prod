use serde::Deserialize;
use surrealdb::engine::remote::ws::Client;

pub type Db = axum::extract::State<surrealdb::Surreal<Client>>;

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application_port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseSettings {
    pub port: u16,
    pub name: String,
    pub host: String,
    pub username: String,
    pub password: String,
    pub namespace: String,
}

impl DatabaseSettings {
    /// Returns a connection string for our database
    /// `Surreal::new<Ws>("<host>:<port>")`
    #[must_use]
    pub fn connection_string(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

#[allow(clippy::module_name_repetitions)]
pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    // Initialize our configuration reader
    let settings = config::Config::builder()
        .add_source(config::File::new(
            "configuration.yaml",
            config::FileFormat::Yaml,
        ))
        .build()?;

    // Try to convert the configuration values it read into
    // our Settings type
    settings.try_deserialize::<Settings>()
}
