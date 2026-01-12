use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(utoipa::ToSchema))]
pub struct UserDTO {
    pub id: i32,
    pub username: String,
    pub password: String,
}
#[cfg(feature = "server")]
impl From<crate::entities::users::Model> for UserDTO {
    fn from(value: crate::entities::users::Model) -> Self {
        UserDTO {
            id: value.id,
            username: value.username,
            password: value.password,
        }
    }
}
#[derive(Deserialize)]
#[cfg_attr(feature = "server", derive(utoipa::ToSchema, utoipa::IntoParams))]
pub struct UpsertUser {
    pub id: Option<i32>,
    pub username: Option<String>,
    #[cfg_attr(feature = "server", into_params(ignore))]
    pub password: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "server", derive(utoipa::ToSchema, sea_orm::FromQueryResult))]
pub struct ReadUser {
    pub id: i32,
    pub username: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[cfg_attr(feature = "server", derive(utoipa::ToSchema))]
pub struct ReqUser {
    pub username: String,
    pub password: String,
    pub save_id: Option<bool>,
}

#[cfg_attr(feature = "server", derive(utoipa::ToSchema))]
#[derive(Deserialize, Serialize, Debug)]
pub struct Tokens {
    pub jwt: String,
    pub refresh: String,
    pub user_info: ReadUser,
}

#[cfg_attr(feature = "server", derive(utoipa::ToSchema))]
#[derive(Deserialize, Serialize)]
pub struct TokensUserId {
    pub jwt: String,
    pub refresh: String,
    pub username: String,
}
