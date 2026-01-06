use axum::{
    Json, Router,
    extract::{Query, State},
};
use reqwest::StatusCode;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, EntityTrait, QueryFilter,
    TryIntoModel,
};
use serde::Deserialize;
use utoipa::{
    IntoParams, Modify, OpenApi, ToSchema,
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
};
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_redoc::{Redoc, Servable as RedocServable};
use utoipa_scalar::{Scalar, Servable};

use crate::{
    entities::users,
    utils::{errors::AppError, hash::hash_password},
};

#[derive(Deserialize, ToSchema, IntoParams)]
struct UpsertUser {
    id: Option<i32>,
    username: Option<String>,
    #[into_params(ignore)]
    password: Option<String>,
}

impl UpsertUser {
    async fn update_user(self, db: &DatabaseConnection) -> Result<users::Model, AppError> {
        self.id.ok_or("can't found update target")?;
        self.create_user(db).await
    }
    async fn create_user(mut self, db: &DatabaseConnection) -> Result<users::Model, AppError> {
        self.password = Some(hash_password(
            &self.password.ok_or("password is not avalable")?,
        )?);

        self.create_user_nonhash(db).await
    }
    async fn create_user_nonhash(self, db: &DatabaseConnection) -> Result<users::Model, AppError> {
        let new_user = users::ActiveModel::try_from(self)?;
        // let new_user: users::ActiveModel = self.try_into()?;
        // 업데이트일수도 있으므로
        Ok(new_user.save(db).await?.try_into_model()?)
    }

    async fn get_users(self, db: &DatabaseConnection) -> Result<Vec<users::Model>, AppError> {
        let mut condition = Condition::all();
        if let Some(id) = self.id {
            condition = condition.add(users::Column::Id.eq(id));
        }
        if let Some(username) = self.username {
            condition = condition.add(users::Column::Username.like(format!("%{}%", username)));
        }

        Ok(users::Entity::find().filter(condition).all(db).await?)
    }

    async fn delete_user(self, db: &DatabaseConnection) -> Result<StatusCode, AppError> {
        let id = self.id.ok_or("can't found delete target")?;
        let res = users::Entity::delete_by_id(id)
            .exec(db)
            .await?
            .rows_affected;

        Ok(if res == 0 {
            Err("target delete faild")
        } else {
            Ok(StatusCode::NO_CONTENT)
        }?)
    }
}

impl TryFrom<UpsertUser> for users::ActiveModel {
    type Error = AppError;
    fn try_from(value: UpsertUser) -> Result<Self, Self::Error> {
        let password = value
            .password
            .ok_or("password not input!")?
            .trim()
            .to_string();
        let username = value
            .username
            .ok_or("user name not input!")?
            .trim()
            .to_string();

        Ok(users::ActiveModel {
            // id가 명시되어있다면 update아니면 create
            id: value
                .id
                .map(sea_orm::ActiveValue::Set)
                .unwrap_or(sea_orm::ActiveValue::NotSet),
            username: sea_orm::ActiveValue::Set(username),
            password: sea_orm::ActiveValue::Set(password),
        })
    }
}

const TAG: &str = "USER";

#[utoipa::path(
    post,
    path = "/post",
    tag = TAG,
    request_body (
        content = UpsertUser,
        content_type = mime::APPLICATION_JSON.as_ref()
    ),
    responses (
        (
            status = StatusCode::CREATED,
            body = users::Model,
        )
    )
)]
async fn post_user(
    State(conn): State<DatabaseConnection>,
    Json(user): Json<UpsertUser>,
) -> Result<(StatusCode, Json<users::Model>), AppError> {
    Ok((StatusCode::CREATED, Json(user.create_user(&conn).await?)))
}

#[utoipa::path(
    get,
    path = "/get",
    tag = TAG,
    params (
        UpsertUser
    ),
    responses (
        (
            status = StatusCode::OK,
            body = Vec<users::Model>,
        )
    )
)]
async fn find_users(
    State(conn): State<DatabaseConnection>,
    Query(user): Query<UpsertUser>,
) -> Result<Json<Vec<users::Model>>, AppError> {
    Ok(Json(user.get_users(&conn).await?))
}

#[utoipa::path(
    put,
    path = "/put",
    tag = TAG,
    request_body (
        content = UpsertUser,
        content_type = mime::APPLICATION_JSON.as_ref()
    ),
    responses (
        (
            status = StatusCode::OK,
            body = users::Model,
        )
    )
)]
async fn put_user(
    State(conn): State<DatabaseConnection>,
    Json(user): Json<UpsertUser>,
) -> Result<Json<users::Model>, AppError> {
    Ok(Json(user.update_user(&conn).await?))
}

#[utoipa::path(
    delete,
    path = "/delete",
    tag = TAG,
    request_body (
        content = UpsertUser,
        content_type = mime::APPLICATION_JSON.as_ref()
    ),
    responses (
        (
            status = StatusCode::NO_CONTENT,
        )
    )
)]
async fn delete_user(
    State(conn): State<DatabaseConnection>,
    Json(user): Json<UpsertUser>,
) -> Result<StatusCode, AppError> {
    Ok(user.delete_user(&conn).await?)
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

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "api_key",
                SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("user_apikey"))),
            );
        }
    }
}

pub(super) fn init_route(base_router: Router, db: DatabaseConnection) -> Router {
    let router = OpenApiRouter::new()
        .routes(routes!(post_user))
        .routes(routes!(find_users))
        .routes(routes!(put_user))
        .routes(routes!(delete_user))
        .with_state(db);

    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        // .merge(base_router)
        .merge(router)
        .split_for_parts();

    let router = router
        .merge(Redoc::with_url("/doc/redoc", api.clone()))
        .merge(Scalar::with_url("/doc/scalar", api));

    let router = base_router.nest("/user", router);
    router
}
