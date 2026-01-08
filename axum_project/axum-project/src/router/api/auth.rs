use axum::extract::State;
use axum::{Json, Router};
use sea_orm::{ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use utoipa::openapi::security::{HttpBuilder, SecurityScheme};
use utoipa::{Modify, OpenApi, ToSchema};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use utoipa_scalar::{Scalar, Servable};

use crate::utils::errors::AppError;

use crate::entities::users;
use crate::utils::hash::verify_password;
use crate::utils::jwt::create_token;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct RequestUser {
    username: String,
    password: String,
}

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
    path = "",
    post,
    tag = TAG,
    request_body(
        content = RequestUser,
        content_type = mime::APPLICATION_JSON.as_ref()
    ),
    responses(
        (status=StatusCode::OK, body=String, example="jwt token")
    )
)]
async fn login(
    State(db): State<DatabaseConnection>,
    Json(request_user): Json<RequestUser>,
) -> Result<String, AppError> {
    let user = users::Entity::find()
        .filter(users::Column::Username.eq(&request_user.username))
        .one(&db)
        .await?;
    match user {
        Some(user) => {
            let _ = verify_password(&request_user.password, &user.password)?;
            Ok(create_token(user.id, user.username)?)
        }
        None => Err(DbErr::RecordNotFound(format!(
            "{} user name not found!",
            request_user.username
        ))
        .into()),
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
    let open_router = OpenApiRouter::new().routes(routes!(login)).with_state(db);

    let (router, login_api) = open_router.split_for_parts();
    let mut api = ApiDoc::openapi();
    api.merge(login_api);

    let router = router.merge(Scalar::with_url("/doc/scalar", api));

    router
}
