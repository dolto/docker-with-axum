use axum::Router;

pub mod auth;
mod user;

pub struct ApiRouters {
    pub auth: Router,
    pub unauth: Router,
}
impl From<(Router, Router)> for ApiRouters {
    fn from(value: (Router, Router)) -> Self {
        ApiRouters {
            auth: value.0,
            unauth: value.1,
        }
    }
}

pub fn init_route(db: sea_orm::DatabaseConnection) -> ApiRouters {
    user::init_route(db.clone()).into()
}
