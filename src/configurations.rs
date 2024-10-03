use secrecy::{ExposeSecret, Secret};
use sqlx::{Connection, Executor, PgConnection, PgPool};

use crate::domain::subscriber_email::SubscriberEmail;

#[derive(serde::Deserialize)]
pub struct Settings {
    pub application: ApplicationSettings,
    pub database: DatabaseSettings,
    pub email_client: EmailClientSettings,
}

#[derive(serde::Deserialize)]
pub struct ApplicationSettings {
    pub host: String,
    pub port: u16,
}

#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

#[derive(serde::Deserialize)]
pub struct EmailClientSettings {
    pub base_url: String,
    pub sender_email: String,
    pub auth_token: Secret<String>,
    pub timeout: u64,
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> Secret<String> {
        Secret::new(format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port,
            self.database_name
        ))
    }

    pub fn connection_string_without_name(&self) -> Secret<String> {
        Secret::new(format!(
            "postgres://{}:{}@{}:{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port
        ))
    }

    pub fn pg_connection_pool(&self) -> PgPool {
        let connection_string = self.connection_string();
        PgPool::connect_lazy(connection_string.expose_secret()).unwrap_or_else(|err| {
            panic!(
                "Failed to connect to the datbase connection pool with error {:?}",
                err
            )
        })
    }

    // Use to config newly created testing database
    pub async fn configure_database(&self) {
        // Create a new random database
        let connection_string = self.connection_string_without_name();
        let mut connection = PgConnection::connect(connection_string.expose_secret())
            .await
            .unwrap_or_else(|err| {
                panic!(
                    "Failed to connect to the datbase connection pool with error {:?}",
                    err
                )
            });
        connection
            .execute(format!(r#"CREATE DATABASE "{}""#, self.database_name).as_str())
            .await
            .unwrap_or_else(|err| panic!("Failed to create a random database with err {}", err));

        // Migrate database
        let mut database_connection =
            PgConnection::connect(self.connection_string().expose_secret())
                .await
                .unwrap_or_else(|err| {
                    panic!(
                        "Failed to connect to the datbase connection pool with error {:?}",
                        err
                    )
                });
        sqlx::migrate!("./migrations")
            .run(&mut database_connection)
            .await
            .expect("Failed to migrate the database");
    }
}

impl EmailClientSettings {
    pub fn sender(&self) -> Result<SubscriberEmail, String> {
        SubscriberEmail::parse(self.sender_email.clone())
    }
}

pub fn read_configuration() -> Result<Settings, config::ConfigError> {
    let base_path = std::env::current_dir().expect("Failed to determine the current dir path");
    let config_dir = base_path.join("config");

    let environment: Environment = std::env::var("APP_ENV")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("Failed to parse APP_ENV");
    let env_file = format!("{}.json", environment.as_str());

    let settings = config::Config::builder()
        .add_source(config::File::from(config_dir.join("base.json")))
        .add_source(config::File::from(config_dir.join(env_file)))
        .build()
        .unwrap_or_else(|err| panic!("Cannot read app configurations with error {:?}", err));

    settings.try_deserialize::<Settings>()
}

pub enum Environment {
    Local,
    Production,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "local" => Ok(Environment::Local),
            "production" => Ok(Environment::Production),
            other => Err(format!(
                "{} is not a supported environment. Use either `local` or `production`.",
                other
            )),
        }
    }
}
