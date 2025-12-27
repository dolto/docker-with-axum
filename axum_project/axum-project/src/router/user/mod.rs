use axum::{Router, routing::get};

async fn user() -> &'static str {
    "User get\n"
}

pub fn user_route() -> Router {
    let users_router = Router::new()
        .route("/", get(user))
        .route("/login", get(|| async move { "User Login\n" }));

    users_router
}
