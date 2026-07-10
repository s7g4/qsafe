//! Integration tests for the HTTP auth flow: Argon2id round-trip via
//! register/login, dual-JWT issuance, refresh-token rotation, and
//! query-based WebSocket token authorization.
//!
//! Requires a reachable Postgres instance (see tests/common/mod.rs for the
//! DATABASE_URL default / override).

mod common;

use common::{spawn_app, unique_username};
use serde_json::{json, Value};

fn extract_cookie_value(set_cookie: &str, name: &str) -> Option<String> {
    set_cookie
        .split(';')
        .map(|s| s.trim())
        .find(|s| s.starts_with(&format!("{name}=")))
        .map(|s| s[name.len() + 1..].to_string())
}

#[tokio::test]
async fn register_then_access_protected_route_with_issued_token() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let username = unique_username("alice");

    let res = client
        .post(format!("{}/api/auth/register", app.base_url))
        .json(&json!({
            "username": username,
            "email": format!("{username}@example.com"),
            "password": "correct-horse-battery-staple",
        }))
        .send()
        .await
        .expect("register request failed");

    assert_eq!(res.status(), 200, "register should succeed");
    let body: Value = res.json().await.unwrap();
    assert!(body["success"].as_bool().unwrap());
    let access_token = body["data"]["access_token"].as_str().unwrap().to_string();
    assert!(!access_token.is_empty());

    // Access token from registration must authorize a protected route.
    let res = client
        .get(format!("{}/api/contacts", app.base_url))
        .bearer_auth(&access_token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200, "valid access token should be accepted");

    // Missing/garbage token must be rejected.
    let res = client
        .get(format!("{}/api/contacts", app.base_url))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 401, "missing token should be rejected");

    let res = client
        .get(format!("{}/api/contacts", app.base_url))
        .bearer_auth("not-a-real-token")
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 401, "garbage token should be rejected");
}

#[tokio::test]
async fn login_verifies_argon2id_password_round_trip() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let username = unique_username("bob");
    let password = "a-genuinely-strong-passphrase";

    let res = client
        .post(format!("{}/api/auth/register", app.base_url))
        .json(&json!({
            "username": username,
            "email": format!("{username}@example.com"),
            "password": password,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);

    // Wrong password must fail against the stored Argon2id hash.
    let res = client
        .post(format!("{}/api/auth/login", app.base_url))
        .json(&json!({ "username": username, "password": "totally-wrong-password" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 401, "wrong password should be rejected");

    // Correct password must succeed and round-trip through Argon2id verify.
    let res = client
        .post(format!("{}/api/auth/login", app.base_url))
        .json(&json!({ "username": username, "password": password }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200, "correct password should be accepted");
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["data"]["username"], username);
}

#[tokio::test]
async fn refresh_endpoint_rotates_tokens_and_rejects_missing_cookie() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let username = unique_username("carol");

    let res = client
        .post(format!("{}/api/auth/register", app.base_url))
        .json(&json!({
            "username": username,
            "email": format!("{username}@example.com"),
            "password": "another-strong-passphrase",
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);

    let set_cookie = res
        .headers()
        .get(reqwest::header::SET_COOKIE)
        .expect("register must set refresh_token cookie")
        .to_str()
        .unwrap()
        .to_string();
    let original_refresh = extract_cookie_value(&set_cookie, "refresh_token")
        .expect("refresh_token cookie must be present");

    // No cookie at all -> rejected.
    let res = client
        .post(format!("{}/api/auth/refresh", app.base_url))
        .send()
        .await
        .unwrap();
    assert_eq!(
        res.status(),
        401,
        "refresh without a cookie must be rejected"
    );

    // Claims (and therefore the signed JWT) are only unique to the second
    // (no `jti`), so refreshing within the same wall-clock second as
    // issuance would mint a byte-identical token. Cross a second boundary so
    // the rotation assertion below actually exercises rotation.
    tokio::time::sleep(std::time::Duration::from_millis(1100)).await;

    // Valid refresh cookie -> new access token + rotated refresh token.
    let res = client
        .post(format!("{}/api/auth/refresh", app.base_url))
        .header(
            reqwest::header::COOKIE,
            format!("refresh_token={original_refresh}"),
        )
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200, "valid refresh cookie should succeed");

    let set_cookie = res
        .headers()
        .get(reqwest::header::SET_COOKIE)
        .expect("refresh must issue a rotated refresh_token cookie")
        .to_str()
        .unwrap()
        .to_string();
    let rotated_refresh = extract_cookie_value(&set_cookie, "refresh_token").unwrap();
    assert_ne!(
        original_refresh, rotated_refresh,
        "refresh token must rotate on use"
    );

    let body: Value = res.json().await.unwrap();
    let new_access_token = body["data"]["access_token"].as_str().unwrap().to_string();
    assert!(!new_access_token.is_empty());

    // The new access token must authorize protected routes.
    let res = client
        .get(format!("{}/api/contacts", app.base_url))
        .bearer_auth(&new_access_token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
}

#[tokio::test]
async fn websocket_upgrade_requires_a_valid_query_token() {
    use tokio_tungstenite::connect_async;

    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let username = unique_username("dave");

    let res = client
        .post(format!("{}/api/auth/register", app.base_url))
        .json(&json!({
            "username": username,
            "email": format!("{username}@example.com"),
            "password": "yet-another-strong-passphrase",
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    let access_token = body["data"]["access_token"].as_str().unwrap();

    // Valid token: the WebSocket upgrade must succeed.
    let ok_url = format!("{}?token={}", app.ws_url, access_token);
    let connect_result = connect_async(&ok_url).await;
    assert!(
        connect_result.is_ok(),
        "expected websocket upgrade to succeed with a valid access token, got {:?}",
        connect_result.err()
    );

    // Missing token: the handler rejects with 401 before the upgrade completes.
    let no_token_url = app.ws_url.clone();
    let connect_result = connect_async(&no_token_url).await;
    assert!(
        connect_result.is_err(),
        "expected websocket upgrade to fail without a token"
    );

    // Garbage token: same rejection path.
    let bad_url = format!("{}?token=not-a-real-jwt", app.ws_url);
    let connect_result = connect_async(&bad_url).await;
    assert!(
        connect_result.is_err(),
        "expected websocket upgrade to fail with an invalid token"
    );
}
