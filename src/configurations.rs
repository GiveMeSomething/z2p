use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;

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

    pub fn connection_string_no_db(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}",
            self.username, self.password, self.host, self.port
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

    pub async fn pg_connection_pool_random(&mut self) -> PgPool {
        let mut connection = PgConnection::connect(&self.connection_string_no_db())
            .await
            .expect("Failed to connect to Postgres database");

        // Create a random database name
        self.database_name = Uuid::new_v4().to_string();
        connection
            .execute(format!(r#"CREATE DATABASE "{}""#, self.database_name).as_str())
            .await
            .unwrap_or_else(|err| panic!("Failed to create a random database with err {}", err));

        // Migrate database
        let connection_pool = PgPool::connect(&self.connection_string())
            .await
            .expect("Failed to connect to the database");
        sqlx::migrate!("./migrations")
            .run(&connection_pool)
            .await
            .expect("Failed to migrate the database");

        return connection_pool;
    }
}

pub fn read_configuration() -> Result<Settings, config::ConfigError> {
    let settings = config::Config::builder()
        .add_source(config::File::new("config.json", config::FileFormat::Json))
        .build()
        .unwrap_or_else(|err| panic!("Cannot read app configurations with error {:?}", err));

    settings.try_deserialize::<Settings>()
}
