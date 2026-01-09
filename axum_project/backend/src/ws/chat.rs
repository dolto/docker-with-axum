use axum::{
    Extension, body,
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    http::Response,
};
use futures_util::{SinkExt, StreamExt};

use crate::{utils::jwt::CurrentUser, ws::state::ChatChannel};

#[utoipa::path(
    get,
    path = "/chat",
    tag = super::TAG,
    params(
        ("Connection" = String, Header, example = "Upgrade"),
        ("Upgrade" = String, Header, example = "websocket"),
        ("Sec-WebSocket-Key" = String, Header, example = "dGhlIHNhbXBsZSBub25jZQ=="),
        ("Sec-WebSocket-Version" = i32, Header, example = 13),
    ),
    description = "브라우저 보안 정책상 UI에서 테스트가 불가합니다, 헤더의 예제를 복사해서 curl로 시도하세요\n\n웹소켓 테스트는 https://github.com/vi/websocat을 추천합니다 ",
    security(("api_jwt_token" = []))
)]
pub async fn chat_ws_handler(
    ws: WebSocketUpgrade,
    Extension(user): Extension<CurrentUser>,
    State(chat): State<ChatChannel>,
) -> Response<body::Body> {
    ws.on_upgrade(|socket| chat_socket_handler(socket, chat, user.1))
}

async fn chat_socket_handler(ws: WebSocket, chat: ChatChannel, username: String) {
    let (mut wtx, mut wrx) = ws.split();

    let mut rx = chat.lock().await.subscribe();

    // chat에서 채팅이 오면 데이터를 받음
    let mut sender = async move || {
        while let Ok(msg) = rx.recv().await {
            if wtx.send(msg).await.is_err() {
                break;
            }
        }
    };
    // 클라이언트가 채팅을 보내면 chat으로 전달
    let mut reciver = async move || {
        if chat
            .lock()
            .await
            .send(format!("{} is Connected", username).into())
            .is_err()
        {}
        while let Some(Ok(msg)) = wrx.next().await {
            match msg {
                Message::Close(_) => {
                    break;
                }
                m => {
                    let m = format!("{}: {}", username, m.to_text().unwrap_or_default());
                    if chat.lock().await.send(Message::Text(m.into())).is_err() {
                        break;
                    }
                }
            }
        }

        if chat
            .lock()
            .await
            .send(format!("{} is Disconnected", username).into())
            .is_err()
        {}
    };

    let (_, _) = tokio::join!(sender(), reciver());
}
