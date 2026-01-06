use bcrypt::{hash, verify};

use crate::utils::errors::AppError;

const COST: u32 = 12;

// hasing
pub fn hash_password(password: &str) -> Result<String, AppError> {
    Ok(hash(password, COST)?)
}

// validtaion
pub fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    Ok(verify(password, hash)?)
}
