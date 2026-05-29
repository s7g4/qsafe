//! WebSocket module for real-time messaging

use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::Response,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Serialize, Deserialize)]
pub enum WSMessage {
    Join {
        session_id: String,
    },
    SendMessage {
        content: String,
        recipient_id: String,
    },
    MessageReceived {
        message: String,
        sender_id: String,
    },
    Error {
        message: String,
    },
}

pub type ConnectedClients =
    Arc<Mutex<HashMap<String, futures_util::stream::SplitSink<WebSocket, Message>>>>;

pub async fn handle_websocket(ws: WebSocketUpgrade, clients: ConnectedClients) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, clients))
}

async fn handle_socket(socket: WebSocket, clients: ConnectedClients) {
    let (mut sender, mut receiver) = socket.split();
    let mut user_id: Option<String> = None;

    while let Some(Ok(msg)) = receiver.next().await {
        if let Message::Text(text) = msg {
            if let Ok(ws_msg) = serde_json::from_str::<WSMessage>(&text) {
                match ws_msg {
                    WSMessage::Join { session_id } => {
                        user_id = Some(session_id.clone());
                        // Note: In a real implementation, we'd need to handle sender cloning differently
                        // For now, we'll skip storing in the clients map to avoid ownership issues
                    }
                    WSMessage::SendMessage {
                        content,
                        recipient_id: _,
                    } => {
                        // Handle message sending logic here
                        // For now, just echo back
                        if let Some(user_id) = &user_id {
                            let response = WSMessage::MessageReceived {
                                message: content,
                                sender_id: user_id.clone(),
                            };
                            if let Ok(json) = serde_json::to_string(&response) {
                                let _ = sender.send(Message::Text(json)).await;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    // Remove client when connection closes
    if let Some(id) = user_id {
        clients.lock().await.remove(&id);
    }
}
