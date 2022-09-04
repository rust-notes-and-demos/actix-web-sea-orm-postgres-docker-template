use actix_web::{HttpResponse, Responder, HttpRequest, body::BoxBody, http::StatusCode};
use derive_more::Display;
use serde::Serialize;

#[derive(Serialize)]
struct FormattedResponseWithId {
    message: String,
    id: usize,
}

#[derive(Serialize)]
struct FormattedResponseWithIds {
    message: String,
    ids: Vec<usize>,
}

#[derive(Debug, Display, PartialEq)]
pub enum CustomResponse {
    #[display(fmt = "Created")]
    Created { id: usize },
    #[display(fmt = "Updated")]
    Updated { id: usize },
    #[display(fmt = "Deleted")]
    Deleted { id: usize },
    #[display(fmt = "Bulk Deleted")]
    BulkDeleted { ids: Vec<usize> },
}

impl CustomResponse {
    fn id(&self) -> Option<usize> {
        match self {
            CustomResponse::Created { id } => Some(*id),
            CustomResponse::Updated { id } => Some(*id),
            CustomResponse::Deleted { id } => Some(*id),
            CustomResponse::BulkDeleted { .. } => None,
        }
    }
    fn status_code (&self) -> StatusCode {
        match self {
            CustomResponse::Created { .. } => StatusCode::CREATED,
            CustomResponse::Updated { .. } => StatusCode::OK,
            CustomResponse::Deleted { .. } => StatusCode::OK,
            CustomResponse::BulkDeleted { .. } => StatusCode::OK,
        }
    }
}

impl Responder for CustomResponse {
    type Body = BoxBody;
    fn respond_to(self, _: &HttpRequest) -> HttpResponse {
        if let CustomResponse::BulkDeleted { ids } = &self {
            let response = FormattedResponseWithIds {
                message: self.to_string(),
                ids: ids.to_owned()
            };
            HttpResponse::build(self.status_code()).json(response)
        } else {
            let response = FormattedResponseWithId {
                message: self.to_string(),
                id: self.id().unwrap(),
            };
            HttpResponse::build(self.status_code()).json(response)
        }
    }
}