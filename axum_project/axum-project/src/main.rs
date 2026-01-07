use axum::{Router, routing::get};
use axum_project::database::init_db;
use axum_project::middle::{init_middel_ware, time_out_test};
use axum_project::router::api::{self, auth};
use axum_project::router::hello::*;

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

    let api_routers = api::init_route(db.clone());

    let hello_router = hello_router(db.clone());
    let login_router = auth::init_router(db.clone());

    let no_auth_router = Router::new()
        .nest("/hello", hello_router)
        .nest("/api/auth", login_router)
        .nest("/api", api_routers.unauth)
        .route("/middle/test", get(time_out_test));

    let auth_router = Router::new().nest("/api", api_routers.auth);

    let app = base_router.merge(auth_router);

    let app = init_middel_ware(no_auth_router, app);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
