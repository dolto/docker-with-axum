use std::time::Duration;

use axum::Router;
use reqwest::StatusCode;
use tower_http::{compression::CompressionLayer, timeout::TimeoutLayer, trace::TraceLayer};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

// 요청은 마지막으로 추가한 레이어부터 순서대로 미들웨어를 경우해서 핸들러에 도달함
// 응답은 역순으로 경유해서 응답을 쏨
pub fn init_middel_ware(route: Router) -> Router {
    // 환경변수 RUST_LOG를 감지해서 로그 단계를 나눔
    // Error < Warn < Info < Debug < Trace
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();
    route
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(3),
        ))
        .layer(TraceLayer::new_for_http())
        // 기본 압축은 tower-http피처에 따라 달라짐 full은 gzip
        // 압축은 가장 마지막에 두는걸 추천 (모든 처리 이후 압축)
        .layer(CompressionLayer::new())
}

pub async fn time_out_test() -> String {
    tokio::time::sleep(Duration::from_secs(5)).await;
    "Why?".to_string()
}
