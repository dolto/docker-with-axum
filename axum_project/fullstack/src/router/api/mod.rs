use axum::Router;

use crate::resources::dto::fullstack_extension::AppExtension;

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

    pub fn nest(self, other: Self, url: &str) -> Self {
        Self {
            auth: self.auth.nest(url, other.auth),
            unauth: self.unauth.nest(url, other.unauth),
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

pub fn init_route(aex: AppExtension) -> Router {
    Router::new().nest("/api", user::init_route(aex))
}
