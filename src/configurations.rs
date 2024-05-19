use secrecy::{ExposeSecret, Secret};
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct Settings {
    pub application: ApplicationSettings,
    pub database: DatabaseSettings,
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

    pub fn connection_string_no_db(&self) -> Secret<String> {
        Secret::new(format!(
            "postgres://{}:{}@{}:{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port,
        ))
    }

    pub async fn pg_connection(&self) -> PgConnection {
        let connection_string = self.connection_string();
        PgConnection::connect(connection_string.expose_secret())
            .await
            .unwrap_or_else(|err| panic!("Failed to connect to the datbase with err {:?}", err))
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

    pub async fn pg_connection_pool_random(&mut self) -> PgPool {
        let mut connection = PgConnection::connect(self.connection_string_no_db().expose_secret())
            .await
            .expect("Failed to connect to Postgres database");

        // Create a random database name
        self.database_name = Uuid::new_v4().to_string();
        connection
            .execute(format!(r#"CREATE DATABASE "{}""#, self.database_name).as_str())
            .await
            .unwrap_or_else(|err| panic!("Failed to create a random database with err {}", err));

        // Migrate database
        let connection_pool = PgPool::connect(self.connection_string().expose_secret())
            .await
            .expect("Failed to connect to the database");
        sqlx::migrate!("./migrations")
            .run(&connection_pool)
            .await
            .expect("Failed to migrate the database");

        connection_pool
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
