mod router;

use crate::router::user::*;
use axum::{
    Router,
    routing::{delete, get, post, put},
};

#[tokio::main]
async fn main() {
    let base_router = Router::new().route(
        "/",
        get(|| async move { "Welcome to Axum\n" })
            .post(|| async move { "Post Something!\n" })
            .put(|| async move { "Updating!...\n" })
            .delete(|| async move { "delete someting\n" }),
    );

    let temps_router = Router::new().route("/", get(|| async move { "Temps Get\n" }));

    let api_router = Router::new()
        .route("/", get(|| async move { "Api Get\n" }))
        .nest("/user", user_route())
        .nest("/temps", temps_router);

    let app = base_router.nest("/api", api_router);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
