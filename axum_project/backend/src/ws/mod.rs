mod chat;
mod state;
use axum::{
    extract::{
        WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use sea_orm::DatabaseConnection;
use tracing::debug;
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_scalar::{Scalar, Servable};

use crate::{
    router::api::{ApiRouters, auth::SecurityAddon},
    ws::state::init_state,
};

pub const TAG: &str = "WebSocket";
#[derive(OpenApi)]
#[openapi(
    servers(
        (url = "/ws", description = "WebSocket Api Doc")
    ),
    modifiers(&SecurityAddon),
    tags((name = TAG, description = "WebSocket Api Tag"))
)]
struct ApiDoc;
pub fn init_router(db: DatabaseConnection) -> ApiRouters {
    let auth_router = OpenApiRouter::new()
        .routes(routes!(websocket_handler))
        .routes(routes!(chat::chat_ws_handler))
        .with_state(init_state());

    let unauth_router = OpenApiRouter::new().with_state(db);

    let (auth_router, auth_api) = auth_router.split_for_parts();
    let (unauth_router, unauth_api) = unauth_router.split_for_parts();

    let mut api = ApiDoc::openapi();
    api.merge(auth_api);
    api.merge(unauth_api);

    let unauth_router = unauth_router.merge(Scalar::with_url("/doc/scalar", api));

    ApiRouters {
        auth: auth_router,
        unauth: unauth_router,
    }
    .new_nest("/ws")
}

// curl -i -N \
//   -H "Connection: Upgrade" \
//   -H "Upgrade: websocket" \
//   -H "Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==" \
//   -H "Sec-WebSocket-Version: 13" \
#[utoipa::path(
    get,
    path = "/test",
    tag = TAG,
    params(
        ("Connection" = String, Header, example = "Upgrade"),
        ("Upgrade" = String, Header, example = "websocket"),
        ("Sec-WebSocket-Key" = String, Header, example = "dGhlIHNhbXBsZSBub25jZQ=="),
        ("Sec-WebSocket-Version" = i32, Header, example = 13),
    ),
    description = "브라우저 보안 정책상 UI에서 테스트가 불가합니다, 헤더의 예제를 복사해서 curl로 시도하세요\n\n웹소켓 테스트는 https://github.com/vi/websocat을 추천합니다 ",
    security(("api_jwt_token" = []))
)]
async fn websocket_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handler_socket)
}

async fn handler_socket(ws: WebSocket) {
    // sender, reciver랑 마찬가지로
    // sink에서 데이터를 보내고, stream에서 데이터를 받는다
    let (mut ws_tx, mut ws_rx) = ws.split();

    while let Some(Ok(msg)) = ws_rx.next().await {
        debug!("Message received: {}", msg.to_text().unwrap_or_default());
        ws_tx
            .send(Message::Text(
                format!("Message received: {}", msg.to_text().unwrap_or_default()).into(),
            ))
            .await
            .unwrap_or_default();
    }
}
