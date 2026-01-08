use axum::Router;

pub mod auth;
mod user;

pub struct ApiRouters {
    pub auth: Router,
    pub unauth: Router,
}
impl ApiRouters {
    pub fn new_nest(self, url: &str) -> Self {
        Self {
            auth: Router::new().nest(url, self.auth),
            unauth: Router::new().nest(url, self.unauth),
        }
    }
    pub fn merge(self, other: Self) -> Self {
        Self {
            auth: self.auth.merge(other.auth),
            unauth: self.unauth.merge(other.unauth),
        }
    }
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
    ApiRouters::from(user::init_route(db.clone())).new_nest("/api")
}
