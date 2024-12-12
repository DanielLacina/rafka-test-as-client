//! Example chat application.
//!
//! Run with
//!
//! ```not_rust
//! cargo run -p example-chat
//! ```

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use futures::{
    sink::SinkExt,
    stream::{SplitSink, SplitStream, StreamExt},
};
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

struct AppState {
    channel_sender: broadcast::Sender<String>,
}

#[tokio::main]
async fn main() {
    let (channel_sender, _) = broadcast::channel(1048);
    let app_state = Arc::new(AppState { channel_sender });
    let app = Router::new().route("/", get(handler).with_state(app_state));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn handler(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> Response {
    ws.on_upgrade(|websocket| handle_websocket(websocket, state))
}

async fn handle_websocket(mut websocket: WebSocket, state: Arc<AppState>) {
    let (mut websocket_sender, mut websocket_receiver) = websocket.split();
    let mut channel_receiver = state.channel_sender.subscribe();
    tokio::spawn(handle_receiver(
        websocket_receiver,
        state.channel_sender.clone(),
    ));
    tokio::spawn(handle_sender(websocket_sender, channel_receiver));
}

async fn handle_receiver(
    mut websocket_receiver: SplitStream<WebSocket>,
    channel_sender: broadcast::Sender<String>,
) {
    while let Some(Ok(msg)) = websocket_receiver.next().await {
        channel_sender.send(msg.into_text().unwrap());
    }
}

async fn handle_sender(
    mut websocket_sender: SplitSink<WebSocket, Message>,
    mut channel_receiver: broadcast::Receiver<String>,
) {
    while let Ok(msg) = channel_receiver.recv().await {
        websocket_sender
            .send(Message::Text(format!("{}", msg)))
            .await;
    }
}
