use axum::Json;
use axum::extract::State;
use sea_orm::{ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};

use crate::utils::errors::AppError;

use crate::entities::users;
use crate::utils::hash::verify_password;
use crate::utils::jwt::create_token;

#[derive(Serialize, Deserialize)]
pub struct RequestUser {
    username: String,
    password: String,
}

pub async fn login(
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
            Ok(create_token(user.username.clone())?)
        }
        None => Err(DbErr::RecordNotFound(format!(
            "{} user name not found!",
            request_user.username
        ))
        .into()),
    }
}
