use actix_web::{web, App, HttpServer};
use actix_web::dev::Server;
use std::net::TcpListener;
use sea_orm::DbConn;
use tracing_actix_web::TracingLogger;
use crate::modules::health_check::controller::health_check;
use crate::modules::todo::controller::{create_todo, update_todo, get_list_of_todos, get_todo, bulk_delete_todos, delete_todo};

/// Runs the HTTP server.
pub fn run(listener: TcpListener, db: DbConn) -> Result<Server, std::io::Error> {
    let db = web::Data::new(db);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            // Register your controllers below 👇
            .service(health_check)
            .service(get_list_of_todos)
            .service(get_todo)
            .service(create_todo)
            .service(update_todo)
            .service(delete_todo)
            .service(bulk_delete_todos)
            // Register application-wide shared data below 👇
            .app_data(db.clone()) // 👈 ❗Important: Register the database connection pool
    })
        .listen(listener)?
        .run();
    Ok(server)
}
