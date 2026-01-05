use chrono::Duration;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use lazy_static::lazy_static;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::env;
use tracing::error;

use crate::utils::errors::AppError;

#[derive(Serialize, Deserialize)]
pub struct Claims {
    exp: i64,
    username: String,
}

lazy_static! {
    static ref SECRET_KEY: String = env::var("SECRET_KEY").expect("SECRET_KEY must be set");
}

pub fn create_token(username: String) -> Result<String, AppError> {
    // 현재시간
    let now = chrono::Utc::now();
    // 토큰 만료시간
    let expires_at = now + Duration::hours(1);
    let exp = expires_at.timestamp();
    // 사용자 이름과, 만료시간을 구조체로 저장
    let claims = Claims { exp, username };
    // 기본 헤더와 시크릿 키를 사용하여 암호화 키 객체를 생성
    let token_header = Header::default();
    let key = EncodingKey::from_secret(SECRET_KEY.as_bytes());

    // 토큰 인코딩
    encode(&token_header, &claims, &key).map_err(|e| {
        error!("Error creating token: {:?}", e);
        AppError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "There was an error, please try again later",
        )
    })
}

pub fn validate_token(token: &str) -> Result<Claims, AppError> {
    let binding = token.replace("Bearer ", "");
    let key = DecodingKey::from_secret(SECRET_KEY.as_bytes());
    let validation = Validation::new(jsonwebtoken::Algorithm::HS256);

    let res = decode::<Claims>(&binding, &key, &validation)
        .map_err(|e| e)
        .and_then(|decoded| {
            // if chrono::Utc::now().timestamp() > decoded.claims.exp {
            Ok(decoded.claims)
            // }
        })?;

    Ok(res)
}
