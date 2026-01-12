use axum::extract::State;
use axum::{Json, Router, debug_handler};
use reqwest::StatusCode;
use sea_orm::{ColumnTrait, DatabaseConnection, DbErr, EntityTrait, ModelTrait, QueryFilter};
use shared::dto::user::{ReqUser, Tokens};
use utoipa::openapi::security::{HttpBuilder, SecurityScheme};
use utoipa::{Modify, OpenApi};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use utoipa_scalar::{Scalar, Servable};

use crate::utils::errors::AppError;

use crate::utils::hash::verify_password;
use crate::utils::jwt::{create_token, validate_jwt_token_without_exp, validate_refresh_token};
use shared::entities::{refresh_token, users};

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
    path = "/login",
    post,
    tag = TAG,
    request_body(
        content = ReqUser,
        content_type = mime::APPLICATION_JSON.as_ref()
    ),
    responses(
        (status=StatusCode::OK, body=Tokens, example="jwt token")
    )
)]
#[debug_handler]
async fn login(
    State(db): State<DatabaseConnection>,
    Json(request_user): Json<ReqUser>,
) -> Result<Json<Tokens>, AppError> {
    let user = users::Entity::find()
        .filter(users::Column::Username.eq(&request_user.username))
        .one(&db)
        .await?;

    match user {
        Some(user) => {
            let _ = verify_password(&request_user.password, &user.password)?;
            let (jwt, refresh) = create_token(user.id, user.username.clone(), &db).await?;

            Ok(Json(Tokens {
                jwt,
                refresh,
                user_info: shared::dto::user::ReadUser {
                    id: user.id,
                    username: user.username,
                },
            }))
        }
        None => Err(DbErr::RecordNotFound(format!(
            "{} user name not found!",
            request_user.username
        ))
        .into()),
    }
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
async fn logout(
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
                user_info: shared::dto::user::ReadUser {
                    id: user_id,
                    username,
                },
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

pub fn init_router(db: DatabaseConnection) -> Router {
    let open_router = OpenApiRouter::new()
        .routes(routes!(login))
        .routes(routes!(logout))
        .routes(routes!(refresh))
        .with_state(db);

    let (router, login_api) = open_router.split_for_parts();
    let mut api = ApiDoc::openapi();
    api.merge(login_api);

    let router = router.merge(Scalar::with_url("/doc/scalar", api));

    router
}
