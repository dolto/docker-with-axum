use std::fmt::Debug;

use axum::{Json, response::IntoResponse};
use bcrypt::BcryptError;
use reqwest::{StatusCode, header::ToStrError};
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
    pub fn auth_error() -> Self {
        Self::new(StatusCode::UNAUTHORIZED, "error validating token")
    }
    pub fn any_error() -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, "some thing is wrong!")
    }

    pub fn any_t_error<T: Debug>(e: T) -> Self {
        error!("Any Error: {:?}", e);
        AppError::new(StatusCode::INTERNAL_SERVER_ERROR, "some thing is wrong")
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
impl From<ToStrError> for AppError {
    fn from(value: ToStrError) -> Self {
        error!("Str Error! {:?}", value);
        AppError::any_error()
    }
}
impl From<BcryptError> for AppError {
    fn from(value: BcryptError) -> Self {
        error!("BcryptError! {:?}", value);
        AppError::new(StatusCode::UNAUTHORIZED, "Id or Pass is wrong!")
    }
}
impl From<&str> for AppError {
    fn from(value: &str) -> Self {
        error!("String Error! {:?}", value);
        AppError::new(StatusCode::UNAUTHORIZED, value)
    }
}
impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        (self.code, Json(self.message)).into_response()
    }
}
