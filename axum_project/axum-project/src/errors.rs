use axum::{Json, RequestPartsExt, response::IntoResponse};
use reqwest::StatusCode;
use sea_orm::DbErr;

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
}

impl From<DbErr> for AppError {
    fn from(_: DbErr) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, DB_ERR_MESSAGE)
    }
}
impl From<reqwest::Error> for AppError {
    fn from(value: reqwest::Error) -> Self {
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Request Error! {:?}", value),
        )
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        (self.code, Json(self.message)).into_response()
    }
}
