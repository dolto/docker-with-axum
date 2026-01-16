#[cfg(feature = "server")]
use crate::resources::dto::fullstack_extension::AppExtension;

pub mod error_layout;
pub mod login;

#[cfg(feature = "server")]
pub fn init_router(aex: AppExtension) -> axum::Router {
    let res = axum::Router::new();
    let login_router = login::init_router(aex.clone());
    let res = res.nest("/front", login_router);

    res
}
