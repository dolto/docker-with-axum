use std::sync::Arc;

use axum::extract::{FromRef, ws::Message};
use tokio::sync::{
    Mutex,
    broadcast::{self, Sender},
};

pub type ChatChannel = Arc<Mutex<Sender<Message>>>;

#[derive(FromRef, Clone, Debug)]
pub struct WsState {
    pub channel: ChatChannel,
}

pub fn init_state() -> WsState {
    let (sender, _) = broadcast::channel(32);
    WsState {
        channel: Arc::new(Mutex::new(sender)),
    }
}
