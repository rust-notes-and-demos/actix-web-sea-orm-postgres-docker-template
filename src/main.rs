use lib::core::config::get_config;
use lib::core::startup::run;
use lib::core::telemetry::{get_subscriber, init_subscriber};
use migration::{Migrator, MigratorTrait};
use secrecy::ExposeSecret;
use std::env;
use std::net::TcpListener;

const APPLICATION_NAME: &str = "{{project-name}}";

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    let application_telemetry_path = env::var("APPLICATION_TELEMETRY_PATH");

    match application_telemetry_path {
        Ok(path) => {
            match path {
                path if path != "" => {
                    // Set up a subscriber for logging to files, rolling daily
                    let subscriber = get_subscriber(APPLICATION_NAME.to_owned(), "info".to_string(), tracing_appender::rolling::daily(path, "log"));
                    init_subscriber(subscriber);
                },
                _ => {
                    // Set up a subscriber for logging to the terminal -- good for development
                    let subscriber = get_subscriber(APPLICATION_NAME.to_owned(), "info".to_string(), std::io::stdout);
                    init_subscriber(subscriber);
                }
            }
        }
        Err(_) => {
            // Set up a subscriber for logging to the terminal -- good for development
            let subscriber = get_subscriber(APPLICATION_NAME.to_owned(), "info".to_string(), std::io::stdout);
            init_subscriber(subscriber);
        }
    }

    // Read the configuration from the environment.
    let config = get_config().expect("Failed to read configuration.");

    // Create a database connection pool.
    let conn = sea_orm::Database::connect(&*config.database.connection_string().expose_secret())
        .await
        .unwrap();

    // ❗ Automatically migrate the database to the latest version when the application starts up.
    // Comment the below three lines if you prefer manual migration instead.
    if let Err(_) = Migrator::up(&conn, None).await {
        panic!("Failed to run migration.");
    }

    // Create a TCP listener at the configured address.
    let address = format!("127.0.0.1:{}", config.application_port);
    let listener = TcpListener::bind(address)?;

    // Run the App 🚀
    run(listener, conn)?.await
}
