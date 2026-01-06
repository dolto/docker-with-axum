use axum::Router;

pub mod auth;
mod user;

pub fn init_route(base_router: Router, db: sea_orm::DatabaseConnection) -> Router {
    user::init_route(base_router, db.clone())
}
