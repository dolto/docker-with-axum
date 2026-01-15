#[cfg(feature = "server")]
pub mod database;
pub mod front;
#[cfg(feature = "server")]
pub mod middle;
pub mod resources;
#[cfg(feature = "server")]
pub mod router;
pub mod utils;
#[cfg(feature = "server")]
pub mod ws;

use crate::front::app;

// #[tokio::main]
fn main() {
    #[cfg(feature = "server")]
    dioxus::serve(|| async move {
        use axum::Extension;
        use axum::{Router, routing::get};
        use middle::{init_middel_ware, time_out_test};
        use router::api::{self, auth};
        use router::hello::*;

        use crate::resources::dto::fullstack_extension;
        // use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

        // 환경변수 RUST_LOG를 감지해서 로그 단계를 나눔
        // Error < Warn < Info < Debug < Trace
        // Dioxus에서 자동으로 init해줌
        // tracing_subscriber::registry()
        //     .with(fmt::layer())
        //     .with(EnvFilter::from_default_env())
        //     .init();

        // fullstack_extension
        let fulex = fullstack_extension::AppExtension::init().await?;

        let api_routers = api::init_route(fulex.clone());
        let ws_routers = ws::init_router(fulex.clone());
        let hello_router = hello_router(fulex.clone());
        let login_router = auth::init_router(fulex.clone());

        let no_auth_router = Router::new()
            .nest("/hello", hello_router)
            .nest("/api/auth", login_router)
            .merge(api_routers.unauth)
            .merge(ws_routers.unauth)
            .merge(dioxus::server::router(app))
            .route("/middle/test", get(time_out_test));

        let auth_router = Router::new().merge(api_routers.auth).merge(ws_routers.auth);

        let app = Router::new().merge(auth_router);

        let app = init_middel_ware(no_auth_router, app).layer(Extension(fulex));

        Ok(app)
        // let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();

        // axum::serve(listener, app).await.unwrap();
    });

    #[cfg(not(feature = "server"))]
    dioxus::launch(app);
}
