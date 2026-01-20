use crate::resources::dto::fullstack_extension::AppExtension;
use crate::resources::dto::user::{CurrentUser, UserDto};
use crate::utils::jwt::authenticate;
use axum::{
    Extension, Json, Router,
    extract::{Query, State},
};
use axum::{Form, middleware};
use reqwest::StatusCode;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_scalar::{Scalar, Servable};

use crate::{router::api::auth::SecurityAddon, utils::errors::AppError};

const TAG: &str = "USER";

#[cfg_attr(feature = "server", derive(utoipa::IntoParams, utoipa::ToSchema))]
#[derive(Serialize, Deserialize)]
pub struct UserGetReq {
    id: Option<i32>,
    username: Option<String>,
    limit: u64,
    page: u64,
}
impl UserGetReq {
    #[cfg(feature = "server")]
    pub async fn get_users(&self, conn: &DatabaseConnection) -> Result<Vec<UserDto>, AppError> {
        use sea_orm::{EntityTrait, PaginatorTrait, QueryFilter};

        use crate::resources::entities::users;

        if let Some(id) = self.id {
            return Ok(users::Entity::find_by_id(id)
                .all(conn)
                .await?
                .into_iter()
                .map(|v| v.into())
                .collect());
        } else if let Some(username) = &self.username {
            use sea_orm::{ColumnTrait, PaginatorTrait};

            return Ok(users::Entity::find()
                .filter(users::Column::Username.like(format!("%{}%", username)))
                .paginate(conn, self.limit)
                .fetch_page(self.page)
                .await?
                .into_iter()
                .map(|v| v.into())
                .collect());
        }

        Ok(users::Entity::find()
            .paginate(conn, self.limit)
            .fetch_page(self.page)
            .await?
            .into_iter()
            .map(|v| v.into())
            .collect())
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
    Form(user): Form<UserDto>,
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
#[cfg(feature = "server")]
impl UserDeleteReq {
    pub async fn delete_user(&self, conn: &DatabaseConnection) -> Result<StatusCode, AppError> {
        use sea_orm::EntityTrait;

        use crate::resources::entities::users;

        let res = users::Entity::delete_by_id(self.id).exec(conn).await?;

        if res.rows_affected == 0 {
            Ok(StatusCode::NOT_FOUND)
        } else {
            Ok(StatusCode::OK)
        }
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
