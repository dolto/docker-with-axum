// 책에는 안나왔지만 이걸 해야할 것 같은데?
mod database;
mod entities;
mod router;

use crate::{
    database::init_db,
    router::{hello::*, user::*},
};
use axum::{Router, routing::get};

#[tokio::main]
async fn main() {
    let db = init_db().await.unwrap();
    let base_router = Router::new().route(
        "/",
        get(|| async move { "Welcome to Axum\n" })
            .post(|| async move { "Post Something!\n" })
            .put(|| async move { "Updating!...\n" })
            .delete(|| async move { "delete someting\n" }),
    );

    let temps_router = Router::new().route("/", get(|| async move { "Temps Get\n" }));

    let hello_router = hello_router(db.clone());

    let api_router = Router::new()
        .route("/", get(|| async move { "Api Get\n" }))
        .nest("/user", user_route())
        .nest("/temps", temps_router);

    let app = base_router
        // DB연결
        .with_state(db.clone())
        .nest("/api", api_router)
        .nest("/hello", hello_router);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
