use lib::core::config::{get_config, DatabaseSettings};
use lib::core::telemetry::{get_subscriber, init_subscriber};
use migration::{Migrator, MigratorTrait};
use once_cell::sync::Lazy;
use sea_orm::{DbConn, DbErr};
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};
use sqlx::{Connection, Executor, PgConnection};
use std::net::TcpListener;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
struct TodoReq {
    title: String,
    description: String,
    done: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct TodoRes {
    id: usize,
}

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "e2e".to_string();
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    }
});

#[tokio::test]
async fn health_check_works() {
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/health_check", test_app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn add_todo_returns_201() {
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();

    let test_cases = vec![
        r#"{"title": "test1", "description": "test1", "done": false}"#,
        r#"{"title": "test2", "description": "test2", "done": true}"#,
        r#"{"title": "test4", "description": "test4", "done": false, "extra": "stuff"}"#,
    ];

    for test_case in test_cases {
        let response = client
            .post(format!("{}/todos", test_app.address))
            .header("Content-Type", "application/json")
            .body(test_case)
            .send()
            .await
            .expect("Failed to execute request.");

        assert_eq!(201, response.status().as_u16());
    }
}

#[tokio::test]
async fn add_todo_returns_400_for_invalid_data() {
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();

    let test_cases = vec![
        r#"{}"#,
        r#"{"title": "test"}"#,
        r#"{"description": "test"}"#,
        r#"{"done": false}"#,
        r#"{"title": "test", "done": false}"#,
        r#"{"title": "test2", "description": "test2"}"#,
        r#"{"description": "test", "done": false}"#,
    ];

    for test_case in test_cases {
        let response = client
            .post(format!("{}/todos", test_app.address))
            .header("Content-Type", "application/json")
            .body(test_case)
            .send()
            .await
            .expect("Failed to execute request.");

        assert_eq!(400, response.status().as_u16());
    }
}

#[tokio::test]
async fn add_todo_returns_409_if_title_already_exists() {
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .post(format!("{}/todos", test_app.address))
        .header("Content-Type", "application/json")
        .body(r#"{"title": "test1", "description": "test1", "done": false}"#)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(201, response.status().as_u16());

    let response = client
        .post(format!("{}/todos", test_app.address))
        .header("Content-Type", "application/json")
        .body(r#"{"title": "test1", "description": "another test1", "done": false}"#)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(409, response.status().as_u16());
}

#[tokio::test]
async fn get_todos_returns_200() {
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/todos", test_app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());
}

#[tokio::test]
async fn get_todo_by_id_returns_200_if_exists() {
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();

    let todo_req = TodoReq {
        title: "test".to_string(),
        description: "test".to_string(),
        done: false,
    };

    let response = client
        .post(format!("{}/todos", test_app.address))
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&todo_req).unwrap())
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(201, response.status().as_u16());

    let todo_res: TodoRes = response.json().await.unwrap();

    let response = client
        .get(format!("{}/todos/{}", test_app.address, todo_res.id))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());
}

#[tokio::test]
async fn get_todo_by_id_returns_404_if_not_exists() {
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/todos/{}", test_app.address, 1))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(404, response.status().as_u16());
}

#[tokio::test]
async fn update_todo_by_id_returns_200_if_exists() {
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();

    let todo_req = TodoReq {
        title: "test".to_string(),
        description: "test".to_string(),
        done: false,
    };

    let response = client
        .post(format!("{}/todos", test_app.address))
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&todo_req).unwrap())
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(201, response.status().as_u16());

    let todo_res: TodoRes = response.json().await.unwrap();

    let todo_req = TodoReq {
        title: "test2".to_string(),
        description: "test2".to_string(),
        done: true,
    };

    let response = client
        .put(format!("{}/todos/{}", test_app.address, todo_res.id))
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&todo_req).unwrap())
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());
}

#[tokio::test]
async fn update_todo_by_id_returns_404_if_not_exists() {
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();

    let todo_req = TodoReq {
        title: "test".to_string(),
        description: "test".to_string(),
        done: false,
    };

    let response = client
        .put(format!("{}/todos/{}", test_app.address, 1))
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&todo_req).unwrap())
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(404, response.status().as_u16());
}

#[tokio::test]
async fn delete_todo_by_id_returns_200_if_exists() {
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();

    let todo_req = TodoReq {
        title: "test".to_string(),
        description: "test".to_string(),
        done: false,
    };

    let response = client
        .post(format!("{}/todos", test_app.address))
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&todo_req).unwrap())
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(201, response.status().as_u16());

    let todo_res: TodoRes = response.json().await.unwrap();

    let response = client
        .delete(format!("{}/todos/{}", test_app.address, todo_res.id))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());
}

#[tokio::test]
async fn delete_todo_by_id_returns_404_if_not_exists() {
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .delete(format!("{}/todos/{}", test_app.address, 999))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(404, response.status().as_u16());
}

#[tokio::test]
async fn bulk_delete_todos_returns_200_if_all_exist() {
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();

    let todo_req = TodoReq {
        title: "test".to_string(),
        description: "test".to_string(),
        done: false,
    };

    let response = client
        .post(format!("{}/todos", test_app.address))
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&todo_req).unwrap())
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(201, response.status().as_u16());

    let todo_res: TodoRes = response.json().await.unwrap();

    let todo_req = TodoReq {
        title: "test2".to_string(),
        description: "test2".to_string(),
        done: false,
    };

    let response = client
        .post(format!("{}/todos", test_app.address))
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&todo_req).unwrap())
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(201, response.status().as_u16());

    let todo_res2: TodoRes = response.json().await.unwrap();

    let response = client
        .delete(format!("{}/todos", test_app.address))
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&vec![todo_res.id, todo_res2.id]).unwrap())
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());
}

#[tokio::test]
async fn bulk_delete_todos_returns_404_if_some_does_not_exist() {
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();

    let todo_req = TodoReq {
        title: "test".to_string(),
        description: "test".to_string(),
        done: false,
    };

    let response = client
        .post(format!("{}/todos", test_app.address))
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&todo_req).unwrap())
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(201, response.status().as_u16());

    let todo_res: TodoRes = response.json().await.unwrap();

    let todo_req = TodoReq {
        title: "test2".to_string(),
        description: "test2".to_string(),
        done: false,
    };

    let response = client
        .post(format!("{}/todos", test_app.address))
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&todo_req).unwrap())
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(201, response.status().as_u16());

    let todo_res2: TodoRes = response.json().await.unwrap();

    let response = client
        .delete(format!("{}/todos", test_app.address))
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&vec![todo_res.id, todo_res2.id, 999]).unwrap())
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(404, response.status().as_u16());
}

pub struct TestApp {
    pub address: String,
    pub db: DbConn,
}

async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let mut configuration = get_config().expect("Failed to read configuration.");
    configuration.database.database_name = Uuid::new_v4().to_string();
    let db = configure_database(&configuration.database).await;
    let db_clone = get_database_conn(&configuration.database).await;

    let server = lib::core::startup::run(listener, db).expect("Failed to bind address");
    let _ = tokio::spawn(server);
    TestApp {
        address,
        db: db_clone,
    }
}

async fn create_database(config: &DatabaseSettings) -> Result<(), sqlx::Error> {
    let mut connection =
        PgConnection::connect(&config.connection_string_without_db().expose_secret())
            .await
            .expect("Failed to connect to Postgres");
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database.");
    Ok(())
}

async fn setup_migration(db: &DbConn) -> Result<(), DbErr> {
    let _res = Migrator::up(db, None).await?;
    Ok(())
}

pub async fn configure_database(config: &DatabaseSettings) -> DbConn {
    create_database(config)
        .await
        .expect("Failed to create database.");
    let conn = get_database_conn(config).await;

    setup_migration(&conn)
        .await
        .expect("Failed to setup migration.");
    conn
}

pub async fn get_database_conn(config: &DatabaseSettings) -> DbConn {
    let conn = sea_orm::Database::connect(&*config.connection_string().expose_secret())
        .await
        .unwrap();
    conn
}
