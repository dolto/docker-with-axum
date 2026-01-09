use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct UserDTO {
    pub id: i32,
    pub username: String,
    pub password: String,
}
