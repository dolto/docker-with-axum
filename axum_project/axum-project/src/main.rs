use axum::{
    Router,
    routing::{delete, get, post, put},
};

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(|| async move { "Welcome to Axum\n" }))
        .route("/", post(|| async move { "Post Something!\n" }))
        .route("/", put(|| async move { "Updating!...\n" }))
        .route("/", delete(|| async move { "delete someting\n" }));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
