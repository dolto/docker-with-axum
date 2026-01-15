use crate::resources::dto::fullstack_extension::AppExtension;
use crate::resources::dto::user::{ReadUser, UpsertUser, UserDTO};
use crate::resources::entities::users;
use axum::{
    Extension, Json, Router,
    extract::{Query, State},
};
use reqwest::StatusCode;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, DbErr, EntityTrait, QueryFilter,
    TryIntoModel,
};
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_scalar::{Scalar, Servable};

use crate::{
    router::api::auth::SecurityAddon,
    utils::{errors::AppError, hash::hash_password, jwt::CurrentUser},
};

trait UserAction {
    async fn update_user(&mut self, db: &DatabaseConnection) -> Result<UserDTO, AppError>;
    async fn create_user(&mut self, db: &DatabaseConnection) -> Result<UserDTO, AppError>;
    async fn create_user_nonhash(&self, db: &DatabaseConnection) -> Result<UserDTO, AppError>;
    async fn get_users(&self, db: &DatabaseConnection) -> Result<Vec<ReadUser>, AppError>;
    async fn delete_user(self, db: &DatabaseConnection) -> Result<StatusCode, AppError>;
    fn into_active_model(value: &UpsertUser) -> Result<users::ActiveModel, AppError>;
}

impl UserAction for UpsertUser {
    async fn update_user(&mut self, db: &DatabaseConnection) -> Result<UserDTO, AppError> {
        self.id.ok_or("can't found update target")?;
        Ok(self.create_user(db).await?.into())
    }
    async fn create_user(&mut self, db: &DatabaseConnection) -> Result<UserDTO, AppError> {
        self.password = Some(hash_password(
            &self.password.take().ok_or("password is not avalable")?,
        )?);

        self.create_user_nonhash(db).await.into()
    }
    async fn create_user_nonhash(&self, db: &DatabaseConnection) -> Result<UserDTO, AppError> {
        let new_user = UpsertUser::into_active_model(self)?;
        // let new_user: users::ActiveModel = self.try_into()?;
        // 업데이트일수도 있으므로
        Ok(new_user.save(db).await?.try_into_model()?.into())
    }

    async fn get_users(&self, db: &DatabaseConnection) -> Result<Vec<ReadUser>, AppError> {
        let mut condition = Condition::all();
        if let Some(id) = self.id {
            condition = condition.add(users::Column::Id.eq(id));
        }
        if let Some(username) = self.username.as_ref() {
            condition = condition.add(users::Column::Username.like(format!("%{}%", username)));
        }

        Ok(users::Entity::find()
            .filter(condition)
            .into_model::<ReadUser>()
            .all(db)
            .await?)
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
    // id가 명시되어있다면 update아니면 create
    fn into_active_model(value: &UpsertUser) -> Result<users::ActiveModel, AppError> {
        let password = value
            .password
            .as_ref()
            .ok_or("password not input!")?
            .trim()
            .to_string();

        // id가 명시인경우 update고, 그런 경우 username은 필수가 아니다
        if let Some(id) = value.id {
            return Ok(users::ActiveModel {
                // id가 명시되어있다면 update아니면 create
                id: sea_orm::ActiveValue::Set(id),
                username: value
                    .username
                    .as_ref()
                    .map(|v| sea_orm::ActiveValue::Set(v.to_string()))
                    .unwrap_or(sea_orm::ActiveValue::NotSet),
                password: sea_orm::ActiveValue::Set(password),
            });
        }
        let username = value
            .username
            .as_ref()
            .ok_or("user name not input!")?
            .trim()
            .to_string();

        Ok(users::ActiveModel {
            id: sea_orm::ActiveValue::NotSet,
            username: sea_orm::ActiveValue::Set(username),
            password: sea_orm::ActiveValue::Set(password),
        })
    }
}

// 현재 수정하고자 하는 유저가 token과 같은 id를 가지고 있는지 확인
async fn is_edit_user(
    edit_user_id: &CurrentUser,
    target_user: &UpsertUser,
) -> Result<bool, AppError> {
    Ok(*edit_user_id
        == target_user
            .id
            .ok_or(DbErr::RecordNotFound("User not found".to_string()))?)
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
            body = UserDTO,
        )
    )
)]
async fn post_user(
    State(conn): State<DatabaseConnection>,
    Json(mut user): Json<UpsertUser>,
) -> Result<(StatusCode, Json<UserDTO>), AppError> {
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
            body = Vec<ReadUser>,
        )
    ),
    security(
        ("api_jwt_token" = [])
    )
)]
async fn find_users(
    State(conn): State<DatabaseConnection>,
    Query(user): Query<UpsertUser>,
) -> Result<Json<Vec<ReadUser>>, AppError> {
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
            body = UserDTO,
        )
    ),
    security(
        ("api_jwt_token" = [])
    )
)]
async fn put_user(
    State(conn): State<DatabaseConnection>,
    Extension(id): Extension<CurrentUser>,
    Json(mut user): Json<UpsertUser>,
) -> Result<Json<UserDTO>, AppError> {
    // 본인만 변경 가능함
    Ok(if is_edit_user(&id, &user).await? {
        Ok(Json(user.update_user(&conn).await?))
    } else {
        Err(AppError::auth_error())
    }?)
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
    ),
    security(
        ("api_jwt_token" = [])
    )
)]
async fn delete_user(
    State(conn): State<DatabaseConnection>,
    Extension(id): Extension<CurrentUser>,
    Json(user): Json<UpsertUser>,
) -> Result<StatusCode, AppError> {
    Ok(if is_edit_user(&id, &user).await? {
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
pub(super) fn init_route(aex: AppExtension) -> (Router, Router) {
    let auth_router = OpenApiRouter::new()
        .routes(routes!(find_users))
        .routes(routes!(put_user))
        .routes(routes!(delete_user))
        .with_state(aex.db.0.clone());

    // 회원가입은 로그인하지 않아도 할 수 있어야함
    let unauth_router = OpenApiRouter::new()
        .routes(routes!(post_user))
        .with_state(aex.db.0);

    // 각각 문서화
    let (auth_router, auth_api) = auth_router.split_for_parts();
    let (unauth_router, unauth_api) = unauth_router.split_for_parts();

    // 문서화된 api를 하나로 합침
    let mut api = ApiDoc::openapi();
    api.merge(auth_api);
    api.merge(unauth_api);

    // 문서 링크는 unauth로 연결하여 인증필요없이 들어갈 수 있도록함
    let unauth_router = unauth_router.merge(Scalar::with_url("/doc/scalar", api));

    // 여전히 인증이 필요한 라우터와 필요없는 라우터는 나눠져있음
    (
        Router::new().nest("/user", auth_router),
        Router::new().nest("/user", unauth_router),
    )
}
