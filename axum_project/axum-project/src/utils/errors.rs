use axum::{Json, response::IntoResponse};
use reqwest::StatusCode;
use sea_orm::DbErr;
use tracing::error;

use crate::database::DB_ERR_MESSAGE;

pub struct AppError {
    code: StatusCode,
    message: String,
}

impl AppError {
    pub fn new(code: StatusCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    pub fn get_db_error() -> AppError {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, DB_ERR_MESSAGE)
    }
    fn auth_error() -> Self {
        Self::new(StatusCode::UNAUTHORIZED, "error validating token")
    }
}

impl From<DbErr> for AppError {
    fn from(e: DbErr) -> Self {
        error!("Data base Error {:?}", e);
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, DB_ERR_MESSAGE)
    }
}
impl From<reqwest::Error> for AppError {
    fn from(value: reqwest::Error) -> Self {
        error!("Requwest Error {:?}", value);
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Request Error! {:?}", value),
        )
    }
}
impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(value: jsonwebtoken::errors::Error) -> Self {
        error!("JWT Error {:?}", value);
        match value.kind() {
            jsonwebtoken::errors::ErrorKind::InvalidToken
            | jsonwebtoken::errors::ErrorKind::InvalidSignature
            | jsonwebtoken::errors::ErrorKind::ExpiredSignature => AppError::auth_error(),
            _ => AppError::new(StatusCode::INTERNAL_SERVER_ERROR, "token error!"),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        (self.code, Json(self.message)).into_response()
    }
}
