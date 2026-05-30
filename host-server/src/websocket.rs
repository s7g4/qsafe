//! WebSocket module for real-time messaging

use crate::database::Database;
use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::Response,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

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

pub enum RegistryCommand {
    Register {
        user_id: String,
        sender: mpsc::Sender<Message>,
    },
    Deregister {
        user_id: String,
    },
    SendMessage {
        recipient_id: String,
        message: Message,
        response_tx: oneshot::Sender<bool>,
    },
}

#[derive(Clone)]
pub struct WebSocketRegistry {
    cmd_tx: mpsc::Sender<RegistryCommand>,
}

impl WebSocketRegistry {
    pub fn new() -> (Self, WebSocketRegistryActor) {
        let (cmd_tx, cmd_rx) = mpsc::channel(1024);
        (
            Self { cmd_tx },
            WebSocketRegistryActor {
                cmd_rx,
                clients: HashMap::new(),
            },
        )
    }

    pub async fn register(&self, user_id: String, sender: mpsc::Sender<Message>) {
        let _ = self
            .cmd_tx
            .send(RegistryCommand::Register { user_id, sender })
            .await;
    }

    pub async fn deregister(&self, user_id: String) {
        let _ = self.cmd_tx.send(RegistryCommand::Deregister { user_id }).await;
    }

    pub async fn send_message(&self, recipient_id: String, message: Message) -> bool {
        let (response_tx, response_rx) = oneshot::channel();
        if self
            .cmd_tx
            .send(RegistryCommand::SendMessage {
                recipient_id,
                message,
                response_tx,
            })
            .await
            .is_err()
        {
            return false;
        }
        response_rx.await.unwrap_or(false)
    }
}

pub struct WebSocketRegistryActor {
    cmd_rx: mpsc::Receiver<RegistryCommand>,
    clients: HashMap<String, mpsc::Sender<Message>>,
}

impl WebSocketRegistryActor {
    pub async fn run(mut self) {
        while let Some(cmd) = self.cmd_rx.recv().await {
            match cmd {
                RegistryCommand::Register { user_id, sender } => {
                    self.clients.insert(user_id, sender);
                }
                RegistryCommand::Deregister { user_id } => {
                    self.clients.remove(&user_id);
                }
                RegistryCommand::SendMessage {
                    recipient_id,
                    message,
                    response_tx,
                } => {
                    let success = if let Some(sender) = self.clients.get(&recipient_id) {
                        sender.try_send(message).is_ok()
                    } else {
                        false
                    };
                    let _ = response_tx.send(success);
                }
            }
        }
    }
}

pub async fn handle_websocket(
    ws: WebSocketUpgrade,
    registry: Arc<WebSocketRegistry>,
    db: Database,
    user_id: Uuid,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, registry, db, user_id))
}

async fn handle_socket(socket: WebSocket, registry: Arc<WebSocketRegistry>, db: Database, user_id: Uuid) {
    let (mut sender, mut receiver) = socket.split();
    let session_id = user_id.to_string();

    metrics::gauge!("qsafe_active_websocket_connections").increment(1.0);

    // Create a bounded channel for writing to this WebSocket asynchronously
    let (tx, mut rx) = mpsc::channel::<Message>(1024);

    registry.register(session_id.clone(), tx.clone()).await;

    // Deliver buffered offline messages from the database
    match db.get_offline_messages(&user_id).await {
        Ok(offline_msgs) => {
            for o_msg in offline_msgs {
                let delivery = WSMessage::MessageReceived {
                    message: o_msg.content,
                    sender_id: o_msg.sender_id.to_string(),
                };
                if let Ok(json) = serde_json::to_string(&delivery) {
                    let _ = tx.try_send(Message::Text(json));
                }
            }
            let _ = db.clear_offline_messages(&user_id).await;
        }
        Err(e) => {
            tracing::error!(error = %e, "Error fetching offline messages");
        }
    }

    // Spawn a writer task to route messages to the websocket client
    let write_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    while let Some(Ok(msg)) = receiver.next().await {
        if let Message::Text(text) = msg {
            if let Ok(ws_msg) = serde_json::from_str::<WSMessage>(&text) {
                match ws_msg {
                    WSMessage::Join { .. } => {
                        // Ignored, auth is handled at connection now
                    }
                    WSMessage::SendMessage {
                        content,
                        recipient_id,
                    } => {
                        metrics::counter!("qsafe_messages_sent_total").increment(1);
                        let response = WSMessage::MessageReceived {
                            message: content.clone(),
                            sender_id: session_id.clone(),
                        };
                        if let Ok(json) = serde_json::to_string(&response) {
                            // Try sending to active online recipient
                            let delivered = registry
                                .send_message(recipient_id.clone(), Message::Text(json))
                                .await;

                            // Buffer message in the database offline queue if client is offline
                            if !delivered {
                                if let Ok(recipient_uuid) = Uuid::parse_str(&recipient_id) {
                                    if let Err(e) = db
                                        .save_offline_message(&recipient_uuid, &user_id, &content)
                                        .await
                                    {
                                        tracing::error!(error = %e, "Failed to buffer offline message");
                                    } else {
                                        metrics::counter!("qsafe_messages_buffered_total")
                                            .increment(1);
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    // Cleanup when client disconnects
    registry.deregister(session_id.clone()).await;
    write_task.abort();
    metrics::gauge!("qsafe_active_websocket_connections").decrement(1.0);
}
