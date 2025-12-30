// 책에는 안나왔지만 이걸 해야할 것 같은데?
mod entities;
mod router;

use crate::router::{hello::*, user::*};
use axum::{Router, routing::get};
use sea_orm::Database;

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

    let hello_router = hello_router();

    let api_router = Router::new()
        .route("/", get(|| async move { "Api Get\n" }))
        .nest("/user", user_route())
        .nest("/temps", temps_router);

    let app = base_router
        .nest("/api", api_router)
        .nest("/hello", hello_router);

    // 컨테이너 내부에 이미 환경변수 설정이 되어있음
    let database_url = std::env::var("DATABASE_URL").unwrap();
    let conn = Database::connect(&database_url).await.unwrap();

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
