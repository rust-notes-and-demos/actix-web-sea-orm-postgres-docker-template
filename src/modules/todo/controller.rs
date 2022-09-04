use crate::{
    modules::todo::{
        dtos::{create_todo_dto::CreateTodoDto, update_todo_dto::UpdateTodoDto},
        service::{find_todo_by_id, find_todos, insert_todo, update_todo_by_id, delete_todo_by_id, bulk_delete_todos_by_ids},
    },
    shared::{errors::CustomError, responses::CustomResponse},
};
use actix_web::{delete, get, post, put, web, HttpResponse};
use sea_orm::DbConn;
use serde::Deserialize;
use validator::Validate;

#[derive(Deserialize)]
pub struct GetTodosQuery {
    query_string: Option<String>,
    page: Option<usize>,
    items_per_page: Option<usize>,
}

#[get("/todos")]
pub async fn get_list_of_todos(
    conn: web::Data<DbConn>,
    query: web::Query<GetTodosQuery>,
) -> Result<HttpResponse, CustomError> {
    let results = find_todos(
        &conn,
        query.query_string.clone(),
        query.items_per_page,
        query.page,
    )
    .await?;
    Ok(HttpResponse::Ok().json(results))
}

#[get("/todos/{id}")]
pub async fn get_todo(
    conn: web::Data<DbConn>,
    path: web::Path<usize>,
) -> Result<HttpResponse, CustomError> {
    let id = path.into_inner();
    let results = find_todo_by_id(&conn, id).await?;
    Ok(HttpResponse::Ok().json(results))
}

#[post("/todos")]
pub async fn create_todo(
    conn: web::Data<DbConn>,
    create_todo_dto: web::Json<CreateTodoDto>,
) -> Result<CustomResponse, CustomError> {
    if let Err(e) = create_todo_dto.validate() {
        println!("DOES IT WORK?");
        return Err(CustomError::ValidationError { e });
    }
    let title = create_todo_dto.title.clone();
    let description = create_todo_dto.description.clone();
    let done = create_todo_dto.done;
    let resp = insert_todo(&conn, title.as_str(), description.as_str(), done).await?;
    Ok(resp)
}

#[put("/todos/{id}")]
pub async fn update_todo(
    conn: web::Data<DbConn>,
    path: web::Path<usize>,
    update_todo_dto: web::Json<UpdateTodoDto>,
) -> Result<CustomResponse, CustomError> {
    if let Err(e) = update_todo_dto.validate() {
        return Err(CustomError::ValidationError { e });
    }
    let id = path.into_inner();
    let title = update_todo_dto.title.clone();
    let description = update_todo_dto.description.clone();
    let done = update_todo_dto.done;
    let resp = update_todo_by_id(&conn, id, title, description, done).await?;
    Ok(resp)
}

#[delete("/todos/{id}")]
pub async fn delete_todo(conn: web::Data<DbConn>, path: web::Path<usize>) -> Result<CustomResponse, CustomError> {
    let id = path.into_inner();
    let resp = delete_todo_by_id(&conn, id).await?;
    Ok(resp)
}

#[delete("/todos")]
pub async fn bulk_delete_todos(conn: web::Data<DbConn>, ids: web::Json<Vec<usize>>) -> Result<CustomResponse, CustomError> {
    let resp = bulk_delete_todos_by_ids(&conn, ids.into_inner()).await?;
    Ok(resp)
}
