use std::fmt::{Debug, Display};

use axum::{http::HeaderValue, response::IntoResponse};
use bcrypt::BcryptError;
use dioxus::{CapturedError, fullstack::AsStatusCode, server::ServerFnError};
use reqwest::{
    StatusCode,
    header::{LOCATION, SET_COOKIE, ToStrError},
};
use sea_orm::DbErr;
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::database::DB_ERR_MESSAGE;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppError {
    pub code: u16,
    pub message: String,
    pub redirect: Option<String>,
}

impl Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "[Err {}]: {}", self.code, self.message)
    }
}
impl std::error::Error for AppError {}

impl AppError {
    pub fn new(code: StatusCode, message: impl Into<String>, referer: Option<String>) -> Self {
        Self {
            code: code.as_u16(),
            message: message.into(),
            redirect: referer,
        }
    }
    pub fn set_redirection(mut self, path: String) -> Self {
        self.redirect = Some(path);
        self
    }

    pub fn get_db_error() -> AppError {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, DB_ERR_MESSAGE, None)
    }
    pub fn auth_error() -> Self {
        Self::new(
            StatusCode::UNAUTHORIZED,
            "error validating token",
            Some("/".to_string()),
        )
    }
    pub fn any_error() -> Self {
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "some thing is wrong!",
            Some("/".to_string()),
        )
    }

    pub fn any_t_error<T: Debug>(e: T) -> Self {
        error!("Any Error: {:?}", e);
        AppError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "some thing is wrong",
            None,
        )
    }
}

impl From<DbErr> for AppError {
    fn from(e: DbErr) -> Self {
        error!("Data base Error {:?}", e);
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, DB_ERR_MESSAGE, None)
    }
}
impl From<reqwest::Error> for AppError {
    fn from(value: reqwest::Error) -> Self {
        error!("Requwest Error {:?}", value);
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Request Error! {:?}", value),
            None,
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
            _ => AppError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "token error!",
                Some("/".to_string()),
            ),
        }
    }
}
impl From<jsonwebtoken::errors::ErrorKind> for AppError {
    fn from(value: jsonwebtoken::errors::ErrorKind) -> Self {
        match value {
            jsonwebtoken::errors::ErrorKind::InvalidToken
            | jsonwebtoken::errors::ErrorKind::InvalidSignature
            | jsonwebtoken::errors::ErrorKind::ExpiredSignature => AppError::auth_error(),
            _ => AppError::new(StatusCode::INTERNAL_SERVER_ERROR, "token error!", None),
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
        AppError::new(StatusCode::UNAUTHORIZED, "Id or Pass is wrong!", None)
    }
}
impl From<&str> for AppError {
    fn from(value: &str) -> Self {
        error!("String Error! {:?}", value);
        AppError::new(StatusCode::UNAUTHORIZED, value, Some("/".to_string()))
    }
}
impl From<axum::Error> for AppError {
    fn from(value: axum::Error) -> Self {
        error!("Axum Error! {:?}", value);
        AppError::any_error()
    }
}
impl From<ServerFnError> for AppError {
    fn from(value: ServerFnError) -> Self {
        error!("Dioxus Server Error! {:?}", value);
        AppError {
            code: value.as_status_code().as_u16(),
            message: "error".to_string(),
            redirect: None,
        }
    }
}
impl From<CapturedError> for AppError {
    fn from(value: CapturedError) -> Self {
        error!("Dioxus Server Error! {:?}", value);
        AppError::any_error()
    }
}
impl From<anyhow::Error> for AppError {
    fn from(value: anyhow::Error) -> Self {
        error!("Dioxus Server Error! {:?}", value);
        AppError::any_error()
    }
}

impl From<AppError> for ServerFnError {
    fn from(value: AppError) -> Self {
        error!("Dioxus Server Error! {:?}", value);
        ServerFnError::new(value)
    }
}
impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let mut response = axum::response::Response::new(axum::body::Body::empty());

        if let Some(path) = self.redirect {
            *response.status_mut() = StatusCode::SEE_OTHER;
            if let Ok(head) = HeaderValue::from_str(&path) {
                let headers = response.headers_mut();
                headers.insert(LOCATION, head);
            }
        } else if let Ok(code) = StatusCode::from_u16(self.code) {
            *response.status_mut() = code;
        }

        let err_msg_val = format!("err_msg={}; Path=/; HttpOnly", self.message);

        if let Ok(head) = HeaderValue::from_str(&err_msg_val) {
            let headers = response.headers_mut();
            headers.append(SET_COOKIE, head);
        }

        response
    }
}
