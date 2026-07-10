//! Minimal end-to-end demo: connects two authenticated WebSocket clients to a
//! running Q-Safe gateway and sends one message between them. Used to
//! produce the recorded demo GIF in docs/ - not part of the test suite.
//!
//! Usage:
//!   cargo run -p qsafe-backend --example demo_client -- <base_url> <sender_token> <recipient_token> <recipient_user_id>

use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let base_url = &args[1];
    let sender_token = &args[2];
    let recipient_token = &args[3];
    let recipient_id = &args[4];

    let ws_base = base_url.replacen("http://", "ws://", 1);

    let (mut recipient_ws, _) = connect_async(format!("{ws_base}/ws?token={recipient_token}"))
        .await
        .expect("recipient failed to connect");
    let (mut sender_ws, _) = connect_async(format!("{ws_base}/ws?token={sender_token}"))
        .await
        .expect("sender failed to connect");

    // Give the registry a moment to register the recipient before sending.
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    let payload = serde_json::json!({
        "SendMessage": {
            "content": "hello from the Q-Safe demo client - quantum-safe and delivered",
            "recipient_id": recipient_id,
        }
    });
    sender_ws
        .send(Message::Text(payload.to_string()))
        .await
        .expect("failed to send message");
    println!("[alice] sent: hello from the Q-Safe demo client - quantum-safe and delivered");

    if let Some(Ok(Message::Text(received))) = recipient_ws.next().await {
        let parsed: serde_json::Value =
            serde_json::from_str(&received).expect("server sent invalid JSON");
        println!(
            "[bob]   received: {}",
            serde_json::to_string_pretty(&parsed).unwrap()
        );
    } else {
        eprintln!("recipient did not receive a message");
        std::process::exit(1);
    }
}
