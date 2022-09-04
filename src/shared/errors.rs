use actix_web::{error::ResponseError, http::StatusCode, HttpResponse};
use derive_more::{Display, Error};
use serde::Serialize;
use validator::ValidationErrors;

#[derive(Serialize)]
struct FormattedErrorResponse {
    status_code: u16,
    error: String,
    message: String,
}

#[derive(Serialize)]
struct FormattedValidationErrorResponse {
    status_code: u16,
    error: String,
    message: ValidationErrors,
}

#[derive(Debug, Display, Error, PartialEq)]
pub enum CustomError {
    #[display(fmt = "Validation error")]
    ValidationError { e: ValidationErrors },
    #[display(fmt = "Internal server error. Please try again later.")]
    ServerError,
    #[display(fmt = "Bad request")]
    BadRequestWithMsg { message: String },
    #[display(fmt = "Bad request")]
    BadRequest,
    #[display(fmt = "Not found")]
    NotFoundWithMsg { message: String },
    #[display(fmt = "Not found")]
    NotFound,
    #[display(fmt = "Conflict")]
    Conflict,
    #[display(fmt = "Unauthorized")]
    Unauthorized,
}

impl CustomError {
    fn message(&self) -> String {
        match self {
            CustomError::ServerError => "Internal Server Error".to_owned(),
            CustomError::BadRequest => "Bad Request".to_owned(),
            CustomError::BadRequestWithMsg { .. } => "Bad Request".to_owned(),
            CustomError::NotFound => "Not Found".to_owned(),
            CustomError::NotFoundWithMsg { .. } => "Not Found".to_owned(),
            CustomError::Conflict => "Conflict".to_owned(),
            CustomError::Unauthorized => "Unauthorized".to_owned(),
            _ => "".to_owned(),
        }
    }
}

impl ResponseError for CustomError {
    fn status_code(&self) -> StatusCode {
        match *self {
            CustomError::ServerError => StatusCode::INTERNAL_SERVER_ERROR,
            CustomError::ValidationError { .. } => StatusCode::BAD_REQUEST,
            CustomError::BadRequest => StatusCode::BAD_REQUEST,
            CustomError::BadRequestWithMsg { .. } => StatusCode::BAD_REQUEST,
            CustomError::NotFound => StatusCode::NOT_FOUND,
            CustomError::NotFoundWithMsg { .. } => StatusCode::NOT_FOUND,
            CustomError::Conflict => StatusCode::CONFLICT,
            CustomError::Unauthorized => StatusCode::UNAUTHORIZED,
        }
    }

    fn error_response(&self) -> HttpResponse {
        if let CustomError::ValidationError { e } = &self {
            let response = FormattedValidationErrorResponse {
                status_code: self.status_code().as_u16(),
                error: self.message(),
                message: e.to_owned(),
            };
            HttpResponse::build(self.status_code()).json(response)
        } else {
            let response = FormattedErrorResponse {
                status_code: self.status_code().as_u16(),
                error: self.message(),
                message: self.to_string(),
            };
            HttpResponse::build(self.status_code()).json(response)
        }
    }
}