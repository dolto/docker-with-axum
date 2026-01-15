use axum::{
    body::Body,
    http::{HeaderMap, Request},
    middleware::Next,
    response::Response,
};
use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use lazy_static::lazy_static;
use reqwest::header::AUTHORIZATION;
use sea_orm::{DatabaseConnection, EntityTrait, sea_query::OnConflict};
use std::{env, str::FromStr};
use tracing::debug;

use crate::resources::{
    dto::user::{JwtClaims, RefreshClaims},
    entities::refresh_token,
};
use crate::utils::errors::AppError;

#[derive(Clone)]
pub struct CurrentUser(pub i32, pub String);
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

// jwt 시드
lazy_static! {
    static ref SECRET_KEY: String = env::var("SECRET_KEY").expect("SECRET_KEY must be set");
}

pub async fn create_token(
    user_id: i32,
    username: String,
    conn: &DatabaseConnection,
) -> Result<(String, String), AppError> {
    // 현재시간
    let now = chrono::Utc::now();
    // 토큰 만료시간
    let expires_at = now + Duration::minutes(15);
    let exp = expires_at.timestamp() as u64;
    // 사용자 이름과, 만료시간을 구조체로 저장
    let claims = JwtClaims {
        exp,
        user_id,
        username: username.clone(),
    };
    // 기본 헤더와 시크릿 키를 사용하여 암호화 키 객체를 생성
    let token_header = Header::default();
    let key = EncodingKey::from_secret(SECRET_KEY.as_bytes());

    // 토큰 인코딩
    let jwt_res = encode(&token_header, &claims, &key)?;
    let refresh_res = create_refresh(user_id, username, now, conn).await?;
    Ok((jwt_res, refresh_res))
}

pub async fn create_refresh(
    user_id: i32,
    username: String,
    now: DateTime<Utc>,
    conn: &DatabaseConnection,
) -> Result<String, AppError> {
    let exp = now + Duration::days(15);
    let exp = exp.naive_utc();
    // 사용자 이름과, 만료시간을 구조체로 저장
    let claims = RefreshClaims { user_id, username };
    // 기본 헤더와 시크릿 키를 사용하여 암호화 키 객체를 생성
    let token_header = Header::default();
    let key = EncodingKey::from_secret(SECRET_KEY.as_bytes());

    let res = encode(&token_header, &claims, &key)?;

    let active = refresh_token::ActiveModel {
        user_id: sea_orm::ActiveValue::Set(user_id),
        token: sea_orm::ActiveValue::Set(res.clone()),
        expires_at: sea_orm::ActiveValue::Set(exp),
    };
    refresh_token::Entity::insert(active)
        .on_conflict(
            OnConflict::column(refresh_token::Column::Token)
                .update_columns([
                    refresh_token::Column::UserId,
                    refresh_token::Column::ExpiresAt,
                ])
                .to_owned(),
        )
        .exec(conn)
        .await?;

    Ok(res)
}

pub fn validate_jwt_token(token: &str) -> Result<JwtClaims, jsonwebtoken::errors::Error> {
    // 암호를 제외한 부분을 제거
    let binding = token.replace("Bearer ", "");
    let key = DecodingKey::from_secret(SECRET_KEY.as_bytes());
    let mut validation = Validation::new(jsonwebtoken::Algorithm::HS256);
    // 60초가 기본이고, 토큰 만료시간의 유예기간이라 생각하면 된다
    validation.leeway = 60;

    let res = decode::<JwtClaims>(&binding, &key, &validation)
        .map_err(|e| e)
        .and_then(|decoded| Ok(decoded.claims))?;
    // 토큰 만료 검사는 decode안에 validate함수안에서 받는 claims구조체에 exp가 있는지 확인하고 검사한다
    // leeway 60초의 여유가 추가적으로 주어진다 (세팅 가능)

    Ok(res)
}

// 만료된 토큰의 claims를 가져옴
pub fn validate_jwt_token_without_exp(
    token: &str,
) -> Result<JwtClaims, jsonwebtoken::errors::Error> {
    // 암호를 제외한 부분을 제거
    // 클라이언트에서 제거하지 않는다는 가정
    let binding = token.replace("Bearer ", "");
    let key = DecodingKey::from_secret(SECRET_KEY.as_bytes());
    let mut validation = Validation::new(jsonwebtoken::Algorithm::HS256);

    validation.validate_exp = false;

    let res = decode::<JwtClaims>(&binding, &key, &validation)
        .map_err(|e| e)
        .and_then(|decoded| Ok(decoded.claims))?;

    Ok(res)
}

pub fn validate_refresh_token(token: &str) -> Result<RefreshClaims, jsonwebtoken::errors::Error> {
    // 암호를 제외한 부분을 제거
    let binding = token.replace("Bearer ", "");
    let key = DecodingKey::from_secret(SECRET_KEY.as_bytes());
    let mut validation = Validation::new(jsonwebtoken::Algorithm::HS256);
    // 리프레시 토큰은 exp가 없으므로 검사하지 않도록 설정
    validation.validate_exp = false;
    validation.required_spec_claims.remove("exp");

    let res = decode::<RefreshClaims>(&binding, &key, &validation)
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
        let claim = validate_jwt_token(token)?;

        debug!("Authenticated user: {}", claim.user_id);

        // 유저 정보를 건내줌으로서, 현재 로그인된 유저를 알 수 있음
        request
            .extensions_mut()
            .insert(CurrentUser(claim.user_id, claim.username));
        Ok(next.run(request).await)
    } else {
        Err(AppError::auth_error())
    }
}
