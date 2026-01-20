use crate::resources::dto::fullstack_extension::AppExtension;
use crate::resources::dto::user::{CurrentUser, UserDto};
use crate::resources::entities::users;
use crate::utils::jwt::authenticate;
use axum::middleware;
use axum::{
    Extension, Json, Router,
    extract::{Query, State},
};
use reqwest::StatusCode;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, DbErr, EntityTrait, QueryFilter,
    TryIntoModel,
};
use serde::{Deserialize, Serialize};
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_scalar::{Scalar, Servable};

use crate::{
    router::api::auth::SecurityAddon,
    utils::{errors::AppError, hash::hash_password},
};

const TAG: &str = "USER";

#[cfg_attr(feature = "server", derive(utoipa::IntoParams, utoipa::ToSchema))]
#[derive(Serialize, Deserialize)]
pub struct UserGetReq {
    id: Option<i32>,
    username: Option<String>,
}
impl UserGetReq {
    pub async fn get_users(&self, conn: &DatabaseConnection) -> Result<Vec<UserDto>, AppError> {
        Err(AppError::any_t_error("아직 구현되지 않은 함수"))
    }
}
#[utoipa::path(
    get,
    path = "/get",
    tag = TAG,
    params (
        UserGetReq
    ),
    responses (
        (
            status = StatusCode::OK,
            body = Vec<UserGetReq>,
        )
    ),
    security(
        ("api_jwt_token" = [])
    )
)]
async fn find_users(
    State(conn): State<DatabaseConnection>,
    Query(user): Query<UserGetReq>,
) -> Result<Json<Vec<UserDto>>, AppError> {
    Ok(Json(user.get_users(&conn).await?))
}

#[utoipa::path(
    put,
    path = "/put",
    tag = TAG,
    request_body (
        content = UserDto,
        content_type = mime::APPLICATION_JSON.as_ref()
    ),
    responses (
        (
            status = StatusCode::OK,
            body = UserDto,
        )
    ),
    security(
        ("api_jwt_token" = [])
    )
)]
async fn put_user(
    State(conn): State<DatabaseConnection>,
    Extension(id): Extension<CurrentUser>,
    Json(mut user): Json<UserDto>,
) -> Result<Json<UserDto>, AppError> {
    // 본인만 변경 가능함
    Ok(if &id == &user.id {
        Ok(Json(user.update_user(&conn).await?))
    } else {
        Err(AppError::auth_error())
    }?)
}

#[cfg_attr(feature = "server", derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize)]
pub struct UserDeleteReq {
    pub id: i32,
}
impl UserDeleteReq {
    pub async fn delete_user(&self, conn: &DatabaseConnection) -> Result<StatusCode, AppError> {
        Err(AppError::any_t_error("아직 구현되지 않은 함수"))
    }
}

#[utoipa::path(
    delete,
    path = "/delete",
    tag = TAG,
    request_body (
        content = UserDeleteReq,
        content_type = mime::APPLICATION_JSON.as_ref()
    ),
    responses (
        (
            status = StatusCode::NO_CONTENT,
        )
    ),
    security(
        ("api_jwt_token" = [])
    )
)]
async fn delete_user(
    State(conn): State<DatabaseConnection>,
    Extension(id): Extension<CurrentUser>,
    Json(user): Json<UserDeleteReq>,
) -> Result<StatusCode, AppError> {
    Ok(if &id == &user.id {
        Ok(user.delete_user(&conn).await?)
    } else {
        Err(AppError::auth_error())
    }?)
}

#[derive(OpenApi)]
#[openapi(
    servers(
        (url = "/api/user", description = "User API base path")
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = TAG, description = "User handler management API")
    )
)]
pub(super) struct ApiDoc;

// 인증이 필요한 라우터 모음
// 인증이 필요 없는 라우터모음으로 나눔
pub(super) fn init_route(aex: AppExtension) -> Router {
    let auth_router = OpenApiRouter::new()
        .routes(routes!(find_users))
        .routes(routes!(put_user))
        .routes(routes!(delete_user))
        .with_state(aex.db.0.clone())
        // 인증 미들웨어 삽입
        .layer(middleware::from_fn(authenticate));

    // 회원가입은 로그인하지 않아도 할 수 있어야함
    let unauth_router = OpenApiRouter::new().with_state(aex.db.0);

    // 각각 문서화
    let (auth_router, auth_api) = auth_router.split_for_parts();
    let (unauth_router, unauth_api) = unauth_router.split_for_parts();

    // 문서화된 api를 하나로 합침
    let mut api = ApiDoc::openapi();
    api.merge(auth_api);
    api.merge(unauth_api);

    // 문서 링크는 unauth로 연결하여 인증필요없이 들어갈 수 있도록함
    let unauth_router = unauth_router.merge(Scalar::with_url("/doc/scalar", api));

    let router = auth_router.merge(unauth_router);

    Router::new().nest("/user", router)
}
