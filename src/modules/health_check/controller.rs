use actix_web::{get, HttpResponse};

#[get("/health_check")]
pub async fn health_check() -> HttpResponse {
    HttpResponse::Ok().finish()
}
