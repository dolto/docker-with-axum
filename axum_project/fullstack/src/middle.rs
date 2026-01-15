use std::time::Duration;

use axum::{Router, middleware};
use reqwest::{StatusCode, header::AUTHORIZATION};
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer, sensitive_headers::SetSensitiveRequestHeadersLayer,
    timeout::TimeoutLayer, trace::TraceLayer,
};

use crate::utils::jwt::authenticate;

// 요청은 마지막으로 추가한 레이어부터 순서대로 미들웨어를 경우해서 핸들러에 도달함
// 응답은 역순으로 경유해서 응답을 쏨
pub fn init_middel_ware(dont_need_auth_route: Router, need_auth_route: Router) -> Router {
    let router = auth_middel_ware(dont_need_auth_route, need_auth_route);

    // ServiceBuilder는 가장 먼저 추가한 녀석부터 처리
    let middleware = ServiceBuilder::new()
        // 동시에 처리할 수 있는 요청 최대치
        .concurrency_limit(32)
        .layer(SetSensitiveRequestHeadersLayer::new(vec![AUTHORIZATION]))
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(3),
        ))
        // 기본 압축은 tower-http피처에 따라 달라짐 full은 gzip
        // 압축은 가장 마지막에 두는걸 추천 (모든 처리 이후 압축)
        .layer(CompressionLayer::new());
    router.layer(middleware).layer(TraceLayer::new_for_http())
}

// 해당 미들웨어의 이전에 정의된 앤드포인트(라우터)는 인증이 필요함
// 즉 인증이 필요없는 앤드포인트 이후에 넣어야하기때문에 따로 빼야함
fn auth_middel_ware(dont_need_auth_route: Router, need_auth_route: Router) -> Router {
    need_auth_route
        .route_layer(middleware::from_fn(authenticate))
        .merge(dont_need_auth_route)
}

#[cfg(feature = "server")]
pub async fn time_out_test() -> String {
    tokio::time::sleep(Duration::from_secs(5)).await;
    "Why?".to_string()
}
