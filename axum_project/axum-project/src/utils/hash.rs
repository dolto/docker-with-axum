use bcrypt::hash;
use reqwest::StatusCode;
use tracing::error;

use crate::utils::errors::AppError;

const COST: u32 = 12;

pub fn hash_password(password: &str) -> Result<String, AppError> {
    hash(password, COST).map_err(|err| {
        error!("Error hashing password {:?}", err);
        AppError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Error securing password!",
        )
    })
}
