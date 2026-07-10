//! Shared harness for integration tests: boots the real Q-Safe axum app
//! (real Postgres, real Argon2id/JWT auth, Mock HSM) on an ephemeral port.

use qsafe_backend::app::{build_router, AppState};
use qsafe_backend::config::Config;
use uuid::Uuid;

#[allow(dead_code)] // not every test binary that includes this module uses every field
pub struct TestApp {
    pub base_url: String,
    pub ws_url: String,
}

fn test_database_url() -> String {
    std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5439/qsafe_test".to_string())
}

/// Spawns a fresh instance of the app on a random loopback port, backed by the
/// shared test Postgres database. Each test uses unique usernames/emails, so
/// tests can run concurrently against the same database.
pub async fn spawn_app() -> TestApp {
    let config = Config {
        database_url: test_database_url(),
        jwt_secret: "integration-test-jwt-secret-do-not-use-in-prod".to_string(),
        port: 0,
        hsm_mock: true,
        hsm_port: None,
        cors_origin: "http://localhost:3000".to_string(),
        db_max_connections: 5,
        tls_cert_path: None,
        tls_key_path: None,
    };

    let state = AppState::build(&config).await.expect(
        "failed to build AppState against test database - is the test Postgres container running?",
    );
    let app = build_router(state, &config.cors_origin, None);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("failed to bind ephemeral port");
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .unwrap();
    });

    TestApp {
        base_url: format!("http://{}", addr),
        ws_url: format!("ws://{}/ws", addr),
    }
}

pub fn unique_username(prefix: &str) -> String {
    format!("{prefix}_{}", &Uuid::new_v4().simple().to_string()[..12])
}
