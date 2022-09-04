use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;
use std::env;

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SslMode {
    Disable,
    Allow,
    Prefer,
    Require,
    VerifyCa,
    VerifyFull
}

#[derive(Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application_port: u16,
}

#[derive(Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    pub port: u16,
    pub host: String,
    pub database_name: String,
    pub ssl_mode: SslMode,
}

impl DatabaseSettings {
    pub fn parse_ssl_mode(&self) -> String {
        match self.ssl_mode {
            SslMode::Disable => "disable".to_owned(),
            SslMode::Allow => "allow".to_owned(),
            SslMode::Prefer => "prefer".to_owned(),
            SslMode::Require => "require".to_owned(),
            SslMode::VerifyCa => "verify-ca".to_owned(),
            SslMode::VerifyFull => "verify-full".to_owned(),
        }
    }
    pub fn connection_string(&self) -> Secret<String> {
        Secret::new(format!(
            "postgres://{}:{}@{}:{}/{}?sslmode={}",
            self.username, self.password.expose_secret(), self.host, self.port, self.database_name, self.parse_ssl_mode()
        ))
    }

    pub fn connection_string_without_db(&self) -> Secret<String> {
        Secret::new(format!(
            "postgres://{}:{}@{}:{}?sslmode={}",
            self.username, self.password.expose_secret(), self.host, self.port, self.parse_ssl_mode()
        ))
    }
}

/// Reads from the .env file and returns the settings in the form of a configuration object.
pub fn get_config() -> Result<Settings, dotenv::Error> {
    dotenv::dotenv().ok();
    let db_host = env::var("DATABASE_HOST").expect("DATABASE_HOST is not set in .env file");
    let db_port = env::var("DATABASE_PORT").expect("DATABASE_PORT is not set in .env file");
    let db_port = db_port.parse().expect("DATABASE_PORT is not a number");
    let db_name = env::var("DATABASE_NAME").expect("DATABASE_NAME is not set in .env file");
    let db_username = env::var("DATABASE_USERNAME").expect("DATABASE_USERNAME is not set in .env file");
    let db_password = env::var("DATABASE_PASSWORD").expect("DATABASE_PASSWORD is not set in .env file");
    let db_ssl_mode = env::var("SSL_MODE").unwrap_or("".to_string());
    let db_ssl_mode = match db_ssl_mode.as_str() {
        "disable" => SslMode::Disable,
        "allow" => SslMode::Allow,
        "prefer" => SslMode::Prefer,
        "require" => SslMode::Require,
        "verify-ca" => SslMode::VerifyCa,
        "verify-full" => SslMode::VerifyFull,
        // if left empty, default to prefer
        "" => SslMode::Prefer,
        other => panic!("SSL_MODE: {} is not a valid value", other),
    };
    let app_port = env::var("APPLICATION_PORT").expect("APPLICATION_PORT is not set in .env file");
    let app_port = app_port.parse().expect("APPLICATION_PORT is not a number");
    Ok(Settings {
        database: DatabaseSettings {
            username: db_username,
            password: Secret::new(db_password),
            port: db_port,
            host: db_host,
            database_name: db_name,
            ssl_mode: db_ssl_mode
        },
        application_port: app_port,
    })
}