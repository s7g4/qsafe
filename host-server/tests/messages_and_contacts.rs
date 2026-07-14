//! Integration tests for the HTTP messages/contacts handlers and the
//! WebSocket offline-message buffering path - previously zero coverage,
//! which is exactly how a real bug slipped through: `messages.session_id`
//! carried a FOREIGN KEY against the never-populated `chat_sessions` table,
//! so every call to POST /api/messages/send failed with a raw 500. Fixed in
//! migrations/0004_drop_dead_session_fk.sql; these tests guard the fix.

mod common;

use common::{spawn_app, unique_username};
use serde_json::{json, Value};

async fn register(base_url: &str, username: &str, password: &str) -> Value {
    reqwest::Client::new()
        .post(format!("{base_url}/api/auth/register"))
        .json(&json!({
            "username": username,
            "email": format!("{username}@example.com"),
            "password": password,
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
}

#[tokio::test]
async fn send_message_then_fetch_it_back() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let alice_name = unique_username("alice_msg");
    let bob_name = unique_username("bob_msg");
    let alice = register(&app.base_url, &alice_name, "alice-message-password").await;
    let bob = register(&app.base_url, &bob_name, "bob-message-password").await;

    let alice_token = alice["data"]["access_token"].as_str().unwrap();
    let alice_id = alice["data"]["user_id"].as_str().unwrap();
    let bob_id = bob["data"]["user_id"].as_str().unwrap();

    let session_id = uuid::Uuid::new_v4();
    let res = client
        .post(format!("{}/api/messages/send", app.base_url))
        .bearer_auth(alice_token)
        .json(&json!({
            "recipient_id": bob_id,
            // base64("hello quantum world")
            "encrypted_content": "aGVsbG8gcXVhbnR1bSB3b3JsZA==",
            "nonce": "bm9uY2U=",
            "session_id": session_id,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(
        res.status(),
        200,
        "sending a message with a fresh, unregistered session_id must succeed - \
         session_id is a client-chosen correlation id, not a foreign-key-checked \
         session record"
    );

    let res = client
        .get(format!("{}/api/messages/{}", app.base_url, bob_id))
        .bearer_auth(alice_token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    let messages = body["data"].as_array().unwrap();
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0]["sender_id"].as_str().unwrap(), alice_id);
    assert_eq!(messages[0]["recipient_id"].as_str().unwrap(), bob_id);
}

#[tokio::test]
async fn get_messages_requires_authentication() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let some_uuid = uuid::Uuid::new_v4();

    let res = client
        .get(format!("{}/api/messages/{}", app.base_url, some_uuid))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 401);
}

#[tokio::test]
async fn add_contact_then_it_appears_in_get_contacts() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let alice_name = unique_username("alice_contact");
    let bob_name = unique_username("bob_contact");
    let alice = register(&app.base_url, &alice_name, "alice-contact-password").await;
    let bob = register(&app.base_url, &bob_name, "bob-contact-password").await;

    let alice_token = alice["data"]["access_token"].as_str().unwrap();
    let bob_id = bob["data"]["user_id"].as_str().unwrap();

    // Before adding: contacts list is empty.
    let res = client
        .get(format!("{}/api/contacts", app.base_url))
        .bearer_auth(alice_token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["data"].as_array().unwrap().len(), 0);

    let res = client
        .post(format!("{}/api/contacts/add", app.base_url))
        .bearer_auth(alice_token)
        .json(&json!({ "contact_id": bob_id }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);

    let res = client
        .get(format!("{}/api/contacts", app.base_url))
        .bearer_auth(alice_token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    let contacts = body["data"].as_array().unwrap();
    assert_eq!(contacts.len(), 1);
    assert_eq!(contacts[0]["username"].as_str().unwrap(), bob_name);
}

#[tokio::test]
async fn websocket_message_is_buffered_offline_and_delivered_on_reconnect() {
    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::{connect_async, tungstenite::Message};

    let app = spawn_app().await;

    let alice_name = unique_username("alice_offline");
    let bob_name = unique_username("bob_offline");
    let alice = register(&app.base_url, &alice_name, "alice-offline-password").await;
    let bob = register(&app.base_url, &bob_name, "bob-offline-password").await;

    let alice_token = alice["data"]["access_token"].as_str().unwrap().to_string();
    let bob_token = bob["data"]["access_token"].as_str().unwrap().to_string();
    let bob_id = bob["data"]["user_id"].as_str().unwrap().to_string();

    // Bob is not connected at all yet: alice's send must be buffered rather
    // than dropped.
    let (mut alice_ws, _) = connect_async(format!("{}?token={}", app.ws_url, alice_token))
        .await
        .expect("alice upgrade should succeed");
    let payload = json!({
        "SendMessage": {
            "content": "buffered while you were away",
            "recipient_id": bob_id,
        }
    });
    alice_ws
        .send(Message::Text(payload.to_string()))
        .await
        .unwrap();

    // Give the server a moment to persist the offline message before bob connects.
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;

    let (mut bob_ws, _) = connect_async(format!("{}?token={}", app.ws_url, bob_token))
        .await
        .expect("bob upgrade should succeed");

    let received = tokio::time::timeout(std::time::Duration::from_secs(5), bob_ws.next())
        .await
        .expect("timed out waiting for the buffered offline message")
        .expect("stream ended without delivering the buffered message")
        .expect("websocket error while waiting for the buffered message");

    let Message::Text(text) = received else {
        panic!("expected a text frame carrying the buffered message");
    };
    let parsed: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(
        parsed["MessageReceived"]["message"].as_str().unwrap(),
        "buffered while you were away"
    );
}
