use axum::{
    Form,
    body::Body,
    http::{HeaderMap, Request},
    middleware::Next,
    response::Response,
};
use chrono::Duration;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use lazy_static::lazy_static;
use reqwest::header::AUTHORIZATION;
use serde::{Deserialize, Serialize};
use std::{env, str::FromStr};
use tracing::debug;

use crate::utils::errors::AppError;

#[derive(Serialize, Deserialize)]
pub struct Claims {
    exp: u64,
    user_id: i32,
}

#[derive(Clone, Copy)]
pub struct CurrentUser(pub i32);
impl From<i32> for CurrentUser {
    fn from(value: i32) -> Self {
        CurrentUser(value)
    }
}
impl PartialEq for CurrentUser {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
    fn ne(&self, other: &Self) -> bool {
        self.0 != other.0
    }
}
impl PartialEq<i32> for CurrentUser {
    fn eq(&self, other: &i32) -> bool {
        self.0 == *other
    }
    fn ne(&self, other: &i32) -> bool {
        self.0 != *other
    }
}

lazy_static! {
    static ref SECRET_KEY: String = env::var("SECRET_KEY").expect("SECRET_KEY must be set");
}

pub fn create_token(user_id: i32) -> Result<String, AppError> {
    // 현재시간
    let now = chrono::Utc::now();
    // 토큰 만료시간
    let expires_at = now + Duration::hours(1);
    let exp = expires_at.timestamp() as u64;
    // 사용자 이름과, 만료시간을 구조체로 저장
    let claims = Claims { exp, user_id };
    // 기본 헤더와 시크릿 키를 사용하여 암호화 키 객체를 생성
    let token_header = Header::default();
    let key = EncodingKey::from_secret(SECRET_KEY.as_bytes());

    // 토큰 인코딩
    let res = encode(&token_header, &claims, &key)?;
    Ok(res)
}

pub fn validate_token(token: &str) -> Result<Claims, AppError> {
    // 암호를 제외한 부분을 제거
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
    mut request: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    if let Some(value) = headers.get(AUTHORIZATION) {
        let token = value.to_str()?;
        let claim = validate_token(token)?;
        debug!("Authenticated user: {}", claim.user_id);

        // 유저 정보를 건내줌으로서, 현재 로그인된 유저를 알 수 있음
        request.extensions_mut().insert(CurrentUser(claim.user_id));
        Ok(next.run(request).await)
    } else {
        Err(AppError::any_t_error(String::from_str(
            "Authorization not found",
        )))
    }
}
