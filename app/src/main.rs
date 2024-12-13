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
use rafka_consumer::Consumer;
use rafka_producer::Producer;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::sync::Mutex;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

struct AppState {
    pub connections: Arc<Mutex<Vec<SplitSink<WebSocket, Message>>>>,
    pub broker_sender: broadcast::Sender<String>,
}

#[tokio::main]
async fn main() {
    let (broker_sender, broker_receiver) = broadcast::channel(1048);
    let app_state = Arc::new(AppState {
        connections: Arc::new(Mutex::new(vec![])),
        broker_sender,
    });
    tokio::spawn(run_consumer(app_state.connections.clone()));
    tokio::spawn(run_producer(broker_receiver));
    let app = Router::new().route("/", get(handler).with_state(app_state));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn run_consumer(connections: Arc<Mutex<Vec<SplitSink<WebSocket, Message>>>>) {
    let mut consumer = Consumer::new("127.0.0.1:8000").await.unwrap();
    consumer.subscribe("app".to_string()).await;
    let mut rx = consumer.consume("app".to_string()).await.unwrap();
    while let Some(message) = rx.recv().await {
        let mut locked_connections = connections.lock().await;
        for connection in locked_connections.iter_mut() {
            connection.send(Message::Text(message.clone())).await;
        }
    }
}

async fn run_producer(mut broker_receiver: broadcast::Receiver<String>) {
    let mut producer = Producer::new("127.0.0.1:8000").await.unwrap();
    while let Ok(msg) = broker_receiver.recv().await {
        producer
            .publish("app".to_string(), msg, "default_key".to_string())
            .await;
    }
}

async fn handler(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> Response {
    ws.on_upgrade(|websocket| handle_websocket(websocket, state))
}

async fn handle_websocket(mut websocket: WebSocket, state: Arc<AppState>) {
    let (mut websocket_sender, mut websocket_receiver) = websocket.split();
    let mut locked_connections = state.connections.lock().await;
    locked_connections.push(websocket_sender);
    tokio::spawn(handle_receiver(
        websocket_receiver,
        state.broker_sender.clone(),
    ));
}

async fn handle_receiver(
    mut websocket_receiver: SplitStream<WebSocket>,
    broker_sender: broadcast::Sender<String>,
) {
    while let Some(Ok(msg)) = websocket_receiver.next().await {
        broker_sender.send(msg.into_text().unwrap());
    }
}
