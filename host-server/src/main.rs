use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use qsafe_backend::{
    auth::{AuthService, LoginRequest, RegisterRequest},
    crypto::CryptoEngine,
    database::Database,
    qkd::QKDProtocol,
    qrng::QRNG,
    websocket::handle_websocket,
};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;

#[allow(dead_code)]
struct AppState {
    db: Database,
    auth: AuthService,
    crypto: Arc<Mutex<CryptoEngine>>,
    qkd: Arc<Mutex<QKDProtocol>>,
    qrng: Arc<Mutex<QRNG>>,
    connected_clients: Arc<
        Mutex<
            HashMap<
                String,
                futures_util::stream::SplitSink<
                    axum::extract::ws::WebSocket,
                    axum::extract::ws::Message,
                >,
            >,
        >,
    >,
}

#[derive(Serialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    message: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load and validate environment configuration
    let config = qsafe_backend::config::Config::load()?;

    // Initialize services
    let db = Database::new(&config.database_url).await?;
    db.create_tables().await?;

    let auth = AuthService::new(config.jwt_secret.clone());
    let crypto = Arc::new(Mutex::new(CryptoEngine::new()));
    let qkd = Arc::new(Mutex::new(QKDProtocol::new()));
    let qrng = Arc::new(Mutex::new(QRNG::new()));
    let connected_clients = Arc::new(Mutex::new(HashMap::new()));

    let state = Arc::new(AppState {
        db,
        auth,
        crypto,
        qkd,
        qrng,
        connected_clients,
    });

    let app = Router::new()
        .route("/api/health", get(health_check))
        .route("/api/auth/register", post(register))
        .route("/api/auth/login", post(login))
        .route("/api/messages/:user_id", get(get_messages))
        .route("/api/messages/send", post(send_message))
        .route("/api/contacts", get(get_contacts))
        .route("/api/contacts/add", post(add_contact))
        .route(
            "/ws",
            get(
                |state: State<Arc<AppState>>, ws: axum::extract::ws::WebSocketUpgrade| {
                    handle_websocket(ws, state.connected_clients.clone())
                },
            ),
        )
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = format!("0.0.0.0:{}", config.port);
    println!("Q-Safe Backend Server running on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> Json<ApiResponse<String>> {
    Json(ApiResponse {
        success: true,
        data: Some("Q-Safe Backend is running!".to_string()),
        message: "Health check passed".to_string(),
    })
}

async fn register(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    // Check if user exists
    if state
        .db
        .get_user_by_username(&req.username)
        .await
        .unwrap()
        .is_some()
    {
        return Err(StatusCode::CONFLICT);
    }

    // Generate quantum key pair
    let mut crypto = state.crypto.lock().await;
    let keypair = crypto
        .generate_pq_keypair()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Hash password
    let password_hash = state
        .auth
        .hash_password(&req.password)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Create user
    let user = state
        .db
        .create_user(
            &req.username,
            &req.email,
            &password_hash,
            &keypair.public_key,
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Create JWT token
    let token = state
        .auth
        .create_token(&user.id, &user.username)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let response = serde_json::json!({
        "token": token,
        "user_id": user.id,
        "username": user.username
    });

    Ok(Json(ApiResponse {
        success: true,
        data: Some(response),
        message: "User registered successfully".to_string(),
    }))
}

async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    // Get user
    let user = state
        .db
        .get_user_by_username(&req.username)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Verify password
    if !state
        .auth
        .verify_password(&req.password, &user.password_hash)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Create JWT token
    let token = state
        .auth
        .create_token(&user.id, &user.username)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let response = serde_json::json!({
        "token": token,
        "user_id": user.id,
        "username": user.username
    });

    Ok(Json(ApiResponse {
        success: true,
        data: Some(response),
        message: "Login successful".to_string(),
    }))
}

async fn get_messages(
    State(_state): State<Arc<AppState>>,
    Path(_user_id): Path<String>,
) -> Result<Json<ApiResponse<Vec<serde_json::Value>>>, StatusCode> {
    Ok(Json(ApiResponse {
        success: true,
        data: Some(vec![]),
        message: "Messages retrieved".to_string(),
    }))
}

async fn send_message(
    State(_state): State<Arc<AppState>>,
    Json(_payload): Json<serde_json::Value>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    Ok(Json(ApiResponse {
        success: true,
        data: Some("Message sent".to_string()),
        message: "Message sent successfully".to_string(),
    }))
}

async fn get_contacts(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<Vec<serde_json::Value>>>, StatusCode> {
    Ok(Json(ApiResponse {
        success: true,
        data: Some(vec![]),
        message: "Contacts retrieved".to_string(),
    }))
}

async fn add_contact(
    State(_state): State<Arc<AppState>>,
    Json(_payload): Json<serde_json::Value>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    Ok(Json(ApiResponse {
        success: true,
        data: Some("Contact added".to_string()),
        message: "Contact added successfully".to_string(),
    }))
}
