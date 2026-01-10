use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::entities::users;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct UserDTO {
    pub id: i32,
    pub username: String,
    pub password: String,
}
impl From<users::Model> for UserDTO {
    fn from(value: users::Model) -> Self {
        UserDTO {
            id: value.id,
            username: value.username,
            password: value.password,
        }
    }
}
