use sqlx::{Connection, PgConnection, PgPool};

#[derive(serde::Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub app_port: u16,
}

#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database_name
        )
    }

    pub async fn pg_connection(&self) -> PgConnection {
        let connection_string = self.connection_string();
        return PgConnection::connect(&connection_string)
            .await
            .unwrap_or_else(|err| panic!("Failed to connect to the datbase with err {:?}", err));
    }

    pub async fn pg_connection_pool(&self) -> PgPool {
        let connection_string = self.connection_string();
        PgPool::connect(&connection_string)
            .await
            .unwrap_or_else(|err| {
                panic!(
                    "Failed to connect to the datbase connection pool with error {:?}",
                    err
                )
            })
    }
}

pub fn read_configuration() -> Result<Settings, config::ConfigError> {
    let settings = config::Config::builder()
        .add_source(config::File::new("config.json", config::FileFormat::Json))
        .build()
        .unwrap_or_else(|err| panic!("Cannot read app configurations with error {:?}", err));

    settings.try_deserialize::<Settings>()
}
