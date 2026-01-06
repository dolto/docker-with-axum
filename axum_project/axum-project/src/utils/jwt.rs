use axum::{
    body::Body,
    http::{HeaderMap, Request},
    middleware::Next,
    response::Response,
};
use chrono::Duration;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::{env, str::FromStr};
use tracing::debug;

use crate::utils::errors::AppError;

#[derive(Serialize, Deserialize)]
pub struct Claims {
    exp: u64,
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
    let exp = expires_at.timestamp() as u64;
    // 사용자 이름과, 만료시간을 구조체로 저장
    let claims = Claims { exp, username };
    // 기본 헤더와 시크릿 키를 사용하여 암호화 키 객체를 생성
    let token_header = Header::default();
    let key = EncodingKey::from_secret(SECRET_KEY.as_bytes());

    // 토큰 인코딩
    let res = encode(&token_header, &claims, &key)?;
    Ok(res)
}

pub fn validate_token(token: &str) -> Result<Claims, AppError> {
    let binding = token.replace("Bearer ", "");
    let key = DecodingKey::from_secret(SECRET_KEY.as_bytes());
    let mut validation = Validation::new(jsonwebtoken::Algorithm::HS256);
    // 60초가 기본이고, 토큰 만료시간의 유예기간이라 생각하면 된다
    validation.leeway = 60;

    let res = decode::<Claims>(&binding, &key, &validation)
        .map_err(|e| e)
        .and_then(|decoded| Ok(decoded.claims))?;
    // 토큰 만료 검사는 decode안에 validate함수안에서 받는 claims구조체에 exp가 있는지 확인하고 검사한다
    // leeway 60초의 여유가 추가적으로 주어진다 (세팅 가능)

    Ok(res)
}

pub async fn authenticate(
    headers: HeaderMap,
    request: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    if let Some(value) = headers.get("Authorization") {
        let token = value.to_str()?;
        let claim = validate_token(token)?;
        debug!("Authenticated user: {}", claim.username);
        Ok(next.run(request).await)
    } else {
        Err(AppError::any_t_error(String::from_str(
            "Authorization not found",
        )))
    }
}
