use crate::front::page::component::login::OauthUrl;
use crate::resources::dto::fullstack_extension::AppExtension;
use crate::resources::dto::user::{CurrentUser, Tokens, UserCondition, UserDto};
use axum::body::Body;
use axum::extract::{FromRef, Query, State};
use axum::http::{HeaderValue, Response};
use axum::{Extension, Form, Json, Router, debug_handler};
use axum_extra::TypedHeader;
use reqwest::StatusCode;
use reqwest::header::{LOCATION, SET_COOKIE};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, ModelTrait, QueryFilter};
use utoipa::openapi::security::{HttpBuilder, SecurityScheme};
use utoipa::{Modify, OpenApi};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use utoipa_scalar::{Scalar, Servable};

use crate::utils::errors::AppError;

use crate::resources::entities::refresh_token;
use crate::utils::jwt::{create_token, validate_jwt_token_without_exp, validate_refresh_token};

pub struct SecurityAddon;
impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "api_jwt_token",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(utoipa::openapi::security::HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );
        }
    }
}

#[utoipa::path(
    path = "/state_setting",
    post,
    tag = TAG,
    request_body(
        content_type = mime::FORM_DATA.as_ref(),
        content = OauthUrl
    ),
    responses(
        (status = StatusCode::SEE_OTHER)
    )
)]
pub async fn state_setting(Form(oauth): Form<OauthUrl>) -> Result<Response<Body>, AppError> {
    let mut response = Response::new(Body::empty());
    *response.status_mut() = StatusCode::SEE_OTHER;

    let url = format!(
        "{}?client_id={}&response_type={}&state={}&scope=email%20profile%20openid&redirect_uri={}",
        oauth.url,
        urlencoding::encode(&std::env::var("CLIENT_ID")?),
        urlencoding::encode(&oauth.response_type),
        urlencoding::encode(&oauth.state),
        urlencoding::encode(&oauth.redirect_uri)
    );

    let headers = response.headers_mut();
    let state_val = format!(
        "state={}; Path=/; HttpOnly; SameSite=Lax; Max-Age=1000",
        oauth.state
    );
    headers.append(SET_COOKIE, HeaderValue::from_str(&state_val)?);
    headers.insert(LOCATION, HeaderValue::from_str(&url)?);

    Ok(response)
}

#[cfg_attr(feature = "server", derive(utoipa::IntoParams))]
#[derive(serde::Deserialize, Debug)]
pub struct GoogleTokenReq {
    pub state: String,
    pub code: String,
    pub scope: String,
    pub authuser: String,
    pub prompt: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct GoogleTokenRes {
    pub access_token: String,
    pub expires_in: i32,
    pub scope: String,
    pub token_type: String,
    pub id_token: String,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct GoogleClaims {
    pub sub: String,
    pub email: String,
}

async fn get_google_claims(
    reqwest: reqwest::Client,
    req: GoogleTokenReq,
    cookies: axum_extra::headers::Cookie,
) -> Result<GoogleClaims, AppError> {
    let cookie_state = cookies.get("state").ok_or(AppError::any_error())?;

    if cookie_state != req.state {
        return Err(AppError::any_error());
    }

    let params = [
        ("code", req.code),
        ("client_id", std::env::var("CLIENT_ID")?),
        ("client_secret", std::env::var("CLIENT_SECRET_KEY")?),
        ("redirect_uri", std::env::var("LOGIN_REDIRECT")?),
        ("grant_type", "authorization_code".to_string()),
    ];
    let res = reqwest
        .post("https://oauth2.googleapis.com/token")
        .form(&params)
        .send()
        .await?;
    if res.status().is_success() {
        tracing::debug!("res success");
        let token_res = res.json::<GoogleTokenRes>().await?;
        tracing::debug!("token to struct");
        let decoded_data =
            jsonwebtoken::dangerous::insecure_decode::<GoogleClaims>(token_res.id_token)?;

        let data = decoded_data.claims;

        // tracing::debug!("{:#?}", data);
        Ok(data)
    } else {
        // tracing::debug!("err google response: {}", res.text().await?);
        Err(AppError::any_t_error("구글 인증에 실패했습니다"))
    }
}
async fn set_token_cookie(
    user: &UserDto,
    db: &DatabaseConnection,
) -> Result<Response<Body>, AppError> {
    let (jwt, refresh) = create_token(user.id, user.username.clone(), db).await?;

    let mut response = Response::new(Body::empty());
    *response.status_mut() = StatusCode::SEE_OTHER;

    let headers = response.headers_mut();
    let jwt_val = format!("jwt={}; Path=/; HttpOnly", jwt);
    let username_val = format!("username={}; Path=/; HttpOnly", user.username);
    let refresh_val = format!("refresh={}; Path=/; HttpOnly", refresh);
    headers.append(SET_COOKIE, HeaderValue::from_bytes(jwt_val.as_bytes())?);
    headers.append(
        SET_COOKIE,
        HeaderValue::from_bytes(username_val.as_bytes())?,
    );
    headers.append(SET_COOKIE, HeaderValue::from_bytes(refresh_val.as_bytes())?);
    headers.insert(LOCATION, HeaderValue::from_static("/"));

    Ok(response)
}

#[utoipa::path(
    path = "/google_login",
    get,
    tag = TAG,
    params(
        GoogleTokenReq
    ),
    responses(
        (status = StatusCode::OK)
    )
)]
#[debug_handler(state = AuthState)]
pub async fn google_login(
    State(reqwest): State<reqwest::Client>,
    State(db): State<DatabaseConnection>,
    Query(req): Query<GoogleTokenReq>,
    TypedHeader(cookies): TypedHeader<axum_extra::headers::Cookie>,
) -> Result<Response<Body>, AppError> {
    // 시나리오
    // 리디렉션을 통해서 로그인시도가 들어가고, username을 찾아서 정보를 가져옴
    // reqwest로 구글 token을 가져오고, 해당 정보에서 유저 id를 찾아서, 데이터베이스에서 찾음
    // 만약 찾는경우 로그인진행, 찾지 못한다면 회원가입 진행

    let google_token = get_google_claims(reqwest.clone(), req, cookies).await?;

    let user_condition = UserCondition {
        username: Some(google_token.email),
        google: Some(google_token.sub),
        ..Default::default()
    };

    let mut user = UserDto::get_user(&user_condition, &db).await?;
    if user.is_empty() {
        user.push(user_condition.post_user(&db).await?);
    }

    set_token_cookie(&user[0], &db).await
}

#[utoipa::path(
    path = "/google_update",
    post,
    tag = TAG,
    request_body(
        content_type = mime::APPLICATION_JSON.as_ref()
    ),
    responses(
        (status = StatusCode::OK)
    ),
    security(
        ("api_jwt_token" = [])
    )
)]
#[debug_handler(state = AuthState)]
pub async fn google_update(
    State(reqwest): State<reqwest::Client>,
    State(db): State<DatabaseConnection>,
    Extension(id): Extension<CurrentUser>,
    Query(req): Query<GoogleTokenReq>,
    TypedHeader(cookies): TypedHeader<axum_extra::headers::Cookie>,
) -> Result<Response<Body>, AppError> {
    // 시나리오
    // 리디렉션을 통해서 로그인시도가 들어가고, username을 찾아서 정보를 가져옴
    // reqwest로 구글 token을 가져오고, 해당 정보에서 유저 id를 찾아서, 데이터베이스에서 찾음
    // 유저 id와 이미 로그인되어있는 id가 일치한다면, 해당 정보를 user데이터베이스에 갱신
    let google_token = get_google_claims(reqwest.clone(), req, cookies).await?;
    let user_condition = UserCondition {
        username: Some(google_token.email),
        google: Some(google_token.sub.clone()),
        ..Default::default()
    };

    let mut user = UserDto::get_user(&user_condition, &db).await?;

    if user.is_empty() || id != user[0].id {
        return Err(AppError::any_t_error(
            "갱신할 수 있는 유저를 찾을 수 없거나 권한이 없습니다",
        ));
    }

    user[0].google = Some(google_token.sub);
    user[0].clone().update_user(&db).await?;

    set_token_cookie(&user[0], &db).await
}

#[utoipa::path(
    path = "/logout",
    post,
    tag = TAG,
    request_body(
        content = String,
        content_type = mime::TEXT_PLAIN.as_ref()
    ),
    responses(
        (status=StatusCode::OK)
    )
)]
// 리프레시 토큰만 제거, 클라이언트에서 의무적으로 Jwt토큰을 제거해야함
pub async fn logout(
    State(db): State<DatabaseConnection>,
    refresh: String,
) -> Result<StatusCode, AppError> {
    let _ = refresh_token::Entity::delete_by_id(refresh)
        .exec(&db)
        .await?;

    // 삭제를 했는지 안했는지와 관계없음
    Ok(StatusCode::OK)
}

#[utoipa::path(
    path = "/refresh",
    post,
    tag = TAG,
    request_body(
        content = Tokens,
        content_type = mime::APPLICATION_JSON.as_ref()
    ),
    responses(
        (status=StatusCode::OK, body=Tokens, example="jwt, refresh token")
    )
)]
#[debug_handler]
// 거절 이후 만료된 jwt토큰과 refresh토큰을 body로 제공
async fn refresh(
    State(db): State<DatabaseConnection>,
    Json(tokens): Json<Tokens>,
) -> Result<Json<Tokens>, AppError> {
    let jwt_claims = validate_jwt_token_without_exp(&tokens.jwt)?;
    let refresh_claims = validate_refresh_token(&tokens.refresh)?;

    // user_id가 동일해야 DB에 접속함
    if refresh_claims.user_id != jwt_claims.user_id {
        return Err(jsonwebtoken::errors::ErrorKind::InvalidSignature.into());
    }

    let user_id = jwt_claims.user_id;
    let username = jwt_claims.username;

    let token_model = refresh_token::Entity::find()
        .filter(
            refresh_token::Column::Token
                .eq(&tokens.refresh)
                .and(refresh_token::Column::UserId.eq(user_id)),
        )
        .one(&db)
        .await?;

    let now = chrono::Utc::now().naive_utc();
    match token_model {
        Some(model) => {
            if now > model.expires_at {
                return Err(jsonwebtoken::errors::ErrorKind::ExpiredSignature.into());
            }
            model.delete(&db).await?;
            let (jwt, refresh) = create_token(user_id, username.clone(), &db).await?;

            Ok(Json(Tokens {
                jwt,
                refresh,
                user_id,
                username,
            }))
        }
        // Lazy 스케줄러가 제거했을 것임
        None => Err(jsonwebtoken::errors::ErrorKind::ExpiredSignature.into()),
    }
}

// OpenAPI
const TAG: &str = "AUTH";
#[derive(OpenApi)]
#[openapi(
    servers(
        (url = "/api/auth", description = "Login API base path")
    ),
    tags(
        (name = TAG, description = "Get JWT Token")
    )
)]
struct ApiDoc;

#[derive(FromRef, Clone)]
struct AuthState {
    db: DatabaseConnection,
    reqwest: reqwest::Client,
}

pub fn init_router(aex: AppExtension) -> Router {
    let open_router = OpenApiRouter::new()
        .routes(routes!(google_login))
        .routes(routes!(logout))
        .routes(routes!(refresh))
        .routes(routes!(state_setting))
        .with_state(AuthState {
            db: aex.db.0,
            reqwest: aex.reqwest.0,
        });

    let (router, login_api) = open_router.split_for_parts();
    let mut api = ApiDoc::openapi();
    api.merge(login_api);

    let router = router.merge(Scalar::with_url("/doc/scalar", api));

    Router::new().nest("/api/auth", router)
}
