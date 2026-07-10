//! HTTP application wiring for Q-Safe: state, extractors, handlers, and router construction.
//!
//! Pulled out of `main.rs` so integration tests can build the real `Router`
//! (register/login/refresh/websocket-auth) instead of re-testing logic in isolation.

use crate::{
    auth::{AuthService, LoginRequest, RegisterRequest},
    config::Config,
    database::Database,
    error::QSafeError,
    hardware::HsmConnection,
    websocket::{handle_websocket, WebSocketRegistry},
};
use axum::{
    extract::{FromRef, Path, Query, State},
    http::header::{HeaderMap, SET_COOKIE},
    http::{HeaderName, HeaderValue, Method},
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use serde::Serialize;
use std::future::ready;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::{
    request_id::{MakeRequestUuid, SetRequestIdLayer},
    trace::TraceLayer,
};
use uuid::Uuid;

pub struct AppState {
    pub db: Database,
    pub auth: AuthService,
    pub hsm: Arc<Mutex<Box<dyn HsmConnection>>>,
    pub registry: Arc<WebSocketRegistry>,
}

impl AppState {
    /// Wires up all services (DB pool, auth, HSM connection, WS registry) from
    /// config. Spawns the WebSocket registry actor onto the current Tokio runtime.
    pub async fn build(config: &Config) -> Result<Arc<Self>, Box<dyn std::error::Error>> {
        let db = Database::new(&config.database_url, config.db_max_connections).await?;
        let auth = AuthService::new(config.jwt_secret.clone());

        let hsm: Arc<Mutex<Box<dyn HsmConnection>>> = if config.hsm_mock {
            Arc::new(Mutex::new(Box::new(
                crate::hardware::MockHsmConnection::new(),
            )))
        } else {
            let port_name = config.hsm_port.as_deref().unwrap_or("COM3");
            let conn = crate::hardware::PhysicalHsmConnection::new(port_name)?;
            Arc::new(Mutex::new(Box::new(conn)))
        };
        let (registry, registry_actor) = WebSocketRegistry::new();
        let registry = Arc::new(registry);
        tokio::spawn(registry_actor.run());

        Ok(Arc::new(Self {
            db,
            auth,
            hsm,
            registry,
        }))
    }
}

#[derive(Serialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    message: String,
}

pub struct AuthedUser {
    pub id: Uuid,
    pub username: String,
}

#[axum::async_trait]
impl<S> axum::extract::FromRequestParts<S> for AuthedUser
where
    Arc<AppState>: axum::extract::FromRef<S>,
    S: Send + Sync,
{
    type Rejection = QSafeError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let app_state = Arc::<AppState>::from_ref(state);

        let auth_header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .ok_or_else(|| QSafeError::Unauthorized("Missing Authorization header".to_string()))?;

        if !auth_header.starts_with("Bearer ") {
            return Err(QSafeError::Unauthorized(
                "Invalid Authorization header format".to_string(),
            ));
        }

        let token = &auth_header["Bearer ".len()..];
        let user_id = app_state.auth.extract_user_id_from_token(token)?;
        let claims = app_state.auth.verify_token(token)?;

        Ok(AuthedUser {
            id: user_id,
            username: claims.username,
        })
    }
}

/// Builds the full axum `Router` (routes + CORS + tracing + request-id + rate limiting),
/// wired to the given state. Does not bind a listener - callers choose plain HTTP,
/// TLS, or an in-process test server.
pub fn build_router(
    state: Arc<AppState>,
    cors_origin: &str,
    prometheus_handle: Option<metrics_exporter_prometheus::PrometheusHandle>,
) -> Router {
    let x_request_id = HeaderName::from_static("x-request-id");

    // Rate limiter configuration: 10 requests per minute (1 req / 6 sec) with burst of 5
    let governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .per_millisecond(6000)
            .burst_size(5)
            .finish()
            .unwrap(),
    );
    let rate_limiter = GovernorLayer {
        config: governor_conf,
    };

    let auth_routes = Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/refresh", post(refresh))
        .route("/logout", post(logout))
        .layer(rate_limiter);

    #[derive(serde::Deserialize)]
    struct WsQuery {
        token: String,
    }

    let mut app = Router::new()
        .route("/api/health", get(health_check))
        .nest("/api/auth", auth_routes)
        .route("/api/messages/:user_id", get(get_messages))
        .route("/api/messages/send", post(send_message))
        .route("/api/contacts", get(get_contacts))
        .route("/api/contacts/add", post(add_contact))
        .route(
            "/ws",
            get(
                |State(state): State<Arc<AppState>>,
                 Query(query): Query<WsQuery>,
                 ws: axum::extract::ws::WebSocketUpgrade| async move {
                    let user_id = match state.auth.extract_user_id_from_token(&query.token) {
                        Ok(id) => id,
                        Err(_) => return axum::http::StatusCode::UNAUTHORIZED.into_response(),
                    };
                    handle_websocket(ws, state.registry.clone(), state.db.clone(), user_id).await
                },
            ),
        );

    if let Some(handle) = prometheus_handle {
        app = app.route("/metrics", get(move || ready(handle.render())));
    }

    app.layer(
        CorsLayer::new()
            .allow_origin(AllowOrigin::exact(
                cors_origin
                    .parse::<HeaderValue>()
                    .unwrap_or_else(|_| HeaderValue::from_static("http://localhost:3000")),
            ))
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::DELETE,
                Method::OPTIONS,
            ])
            .allow_headers([
                axum::http::header::AUTHORIZATION,
                axum::http::header::CONTENT_TYPE,
            ])
            .allow_credentials(true),
    )
    .layer(TraceLayer::new_for_http())
    .layer(SetRequestIdLayer::new(x_request_id, MakeRequestUuid))
    .with_state(state)
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
) -> Result<(HeaderMap, Json<ApiResponse<serde_json::Value>>), QSafeError> {
    // Input validation
    if req.username.len() < 3 || req.username.len() > 32 {
        return Err(QSafeError::ValidationError(
            "Username must be between 3 and 32 characters".to_string(),
        ));
    }
    if !req
        .username
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_')
    {
        return Err(QSafeError::ValidationError(
            "Username must contain only alphanumeric characters and underscores".to_string(),
        ));
    }
    if !req.email.contains('@') || !req.email.contains('.') {
        return Err(QSafeError::ValidationError(
            "Invalid email address format".to_string(),
        ));
    }
    if req.password.len() < 8 {
        return Err(QSafeError::ValidationError(
            "Password must be at least 8 characters".to_string(),
        ));
    }

    // Check if user exists
    if state
        .db
        .get_user_by_username(&req.username)
        .await?
        .is_some()
    {
        return Err(QSafeError::UserConflict(
            "Username already taken".to_string(),
        ));
    }

    // Generate quantum public key from HSM
    let public_key = {
        let mut hsm = state.hsm.lock().await;
        hsm.send_request(qsafe_common::PacketType::GetPublicKeyReq, &[])?
    };

    // Hash password using Argon2id
    let password_hash = state.auth.hash_password(&req.password)?;

    // Create user
    let user = state
        .db
        .create_user(&req.username, &req.email, &password_hash, &public_key)
        .await?;

    // Create dual tokens
    let access_token = state.auth.create_access_token(&user.id, &user.username)?;
    let refresh_token = state.auth.create_refresh_token(&user.id, &user.username)?;

    // Set refresh token in HttpOnly Cookie
    let mut headers = HeaderMap::new();
    let cookie_value = format!(
        "refresh_token={}; Path=/; HttpOnly; Secure; SameSite=Strict; Max-Age={}",
        refresh_token,
        7 * 24 * 60 * 60 // 7 days in seconds
    );
    headers.insert(
        SET_COOKIE,
        HeaderValue::from_str(&cookie_value)
            .map_err(|_| QSafeError::Internal("Cookie compilation failed".to_string()))?,
    );

    let response = serde_json::json!({
        "access_token": access_token,
        "user_id": user.id,
        "username": user.username
    });

    Ok((
        headers,
        Json(ApiResponse {
            success: true,
            data: Some(response),
            message: "User registered successfully".to_string(),
        }),
    ))
}

async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> Result<(HeaderMap, Json<ApiResponse<serde_json::Value>>), QSafeError> {
    // Get user
    let user = state
        .db
        .get_user_by_username(&req.username)
        .await?
        .ok_or_else(|| QSafeError::Unauthorized("Invalid username or password".to_string()))?;

    // Verify password using Argon2id
    if !state
        .auth
        .verify_password(&req.password, &user.password_hash)?
    {
        return Err(QSafeError::Unauthorized(
            "Invalid username or password".to_string(),
        ));
    }

    // Create dual tokens
    let access_token = state.auth.create_access_token(&user.id, &user.username)?;
    let refresh_token = state.auth.create_refresh_token(&user.id, &user.username)?;

    // Set refresh token in HttpOnly Cookie
    let mut headers = HeaderMap::new();
    let cookie_value = format!(
        "refresh_token={}; Path=/; HttpOnly; Secure; SameSite=Strict; Max-Age={}",
        refresh_token,
        7 * 24 * 60 * 60 // 7 days in seconds
    );
    headers.insert(
        SET_COOKIE,
        HeaderValue::from_str(&cookie_value)
            .map_err(|_| QSafeError::Internal("Cookie compilation failed".to_string()))?,
    );

    let response = serde_json::json!({
        "access_token": access_token,
        "user_id": user.id,
        "username": user.username
    });

    Ok((
        headers,
        Json(ApiResponse {
            success: true,
            data: Some(response),
            message: "Login successful".to_string(),
        }),
    ))
}

async fn refresh(
    State(state): State<Arc<AppState>>,
    headers_in: HeaderMap,
) -> Result<(HeaderMap, Json<ApiResponse<serde_json::Value>>), QSafeError> {
    // Extract cookie manually to avoid external dependencies
    let cookie_header = headers_in
        .get(axum::http::header::COOKIE)
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| QSafeError::Unauthorized("Missing refresh token cookie".to_string()))?;

    let refresh_token = cookie_header
        .split(';')
        .map(|s| s.trim())
        .find(|s| s.starts_with("refresh_token="))
        .map(|s| s["refresh_token=".len()..].to_string())
        .ok_or_else(|| QSafeError::Unauthorized("Missing refresh token cookie".to_string()))?;

    // Verify token
    let claims = state.auth.verify_token(&refresh_token)?;
    if claims.token_type != "refresh" {
        return Err(QSafeError::Unauthorized("Invalid token type".to_string()));
    }

    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| QSafeError::Unauthorized("Invalid user ID in token".to_string()))?;

    // Load user to verify they still exist
    let user = state
        .db
        .get_user_by_id(&user_id)
        .await?
        .ok_or_else(|| QSafeError::Unauthorized("User not found".to_string()))?;

    // Generate rotated tokens
    let access_token = state.auth.create_access_token(&user.id, &user.username)?;
    let new_refresh_token = state.auth.create_refresh_token(&user.id, &user.username)?;

    // Set new refresh token in HttpOnly Cookie
    let mut headers_out = HeaderMap::new();
    let cookie_value = format!(
        "refresh_token={}; Path=/; HttpOnly; Secure; SameSite=Strict; Max-Age={}",
        new_refresh_token,
        7 * 24 * 60 * 60
    );
    headers_out.insert(
        SET_COOKIE,
        HeaderValue::from_str(&cookie_value)
            .map_err(|_| QSafeError::Internal("Cookie compilation failed".to_string()))?,
    );

    let response = serde_json::json!({
        "access_token": access_token,
        "user_id": user.id,
        "username": user.username
    });

    Ok((
        headers_out,
        Json(ApiResponse {
            success: true,
            data: Some(response),
            message: "Token refreshed successfully".to_string(),
        }),
    ))
}

async fn logout() -> Result<(HeaderMap, Json<ApiResponse<String>>), QSafeError> {
    // Clear refresh token cookie by setting Max-Age=0
    let mut headers = HeaderMap::new();
    let cookie_value = "refresh_token=; Path=/; HttpOnly; Secure; SameSite=Strict; Max-Age=0";
    headers.insert(
        SET_COOKIE,
        HeaderValue::from_str(cookie_value)
            .map_err(|_| QSafeError::Internal("Cookie compilation failed".to_string()))?,
    );

    Ok((
        headers,
        Json(ApiResponse {
            success: true,
            data: Some("Logged out".to_string()),
            message: "Logged out successfully".to_string(),
        }),
    ))
}

#[derive(serde::Deserialize)]
struct SendMessagePayload {
    pub recipient_id: Uuid,
    pub encrypted_content: String,
    pub nonce: String,
    pub session_id: Uuid,
}

#[derive(serde::Deserialize)]
struct AddContactPayload {
    pub contact_id: Uuid,
}

async fn get_messages(
    State(state): State<Arc<AppState>>,
    authed_user: AuthedUser,
    Path(target_user_id_str): Path<String>,
) -> Result<Json<ApiResponse<Vec<serde_json::Value>>>, QSafeError> {
    let target_user_id = Uuid::parse_str(&target_user_id_str)
        .map_err(|_| QSafeError::BadRequest("Invalid target user ID".to_string()))?;

    let messages = state
        .db
        .get_messages_between_users(&authed_user.id, &target_user_id, 50)
        .await?;

    let serialized_messages: Vec<serde_json::Value> = messages
        .into_iter()
        .map(|msg| serde_json::to_value(msg).unwrap())
        .collect();

    Ok(Json(ApiResponse {
        success: true,
        data: Some(serialized_messages),
        message: "Messages retrieved".to_string(),
    }))
}

async fn send_message(
    State(state): State<Arc<AppState>>,
    authed_user: AuthedUser,
    Json(payload): Json<SendMessagePayload>,
) -> Result<Json<ApiResponse<String>>, QSafeError> {
    use base64::{engine::general_purpose::STANDARD, Engine as _};

    let encrypted_content = STANDARD
        .decode(&payload.encrypted_content)
        .map_err(|_| QSafeError::BadRequest("Invalid base64 encrypted content".to_string()))?;
    if encrypted_content.len() > 1_048_576 {
        return Err(QSafeError::ValidationError(
            "Encrypted content exceeds 1MB limit".to_string(),
        ));
    }
    let nonce = STANDARD
        .decode(&payload.nonce)
        .map_err(|_| QSafeError::BadRequest("Invalid base64 nonce".to_string()))?;
    if nonce.len() > 128 {
        return Err(QSafeError::ValidationError(
            "Nonce exceeds 128 byte limit".to_string(),
        ));
    }

    state
        .db
        .save_message(
            &authed_user.id,
            &payload.recipient_id,
            &encrypted_content,
            &nonce,
            &payload.session_id,
        )
        .await?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some("Message sent".to_string()),
        message: "Message sent successfully".to_string(),
    }))
}

async fn get_contacts(
    State(state): State<Arc<AppState>>,
    authed_user: AuthedUser,
) -> Result<Json<ApiResponse<Vec<serde_json::Value>>>, QSafeError> {
    let contacts = state.db.get_contacts(&authed_user.id).await?;

    let serialized_contacts: Vec<serde_json::Value> = contacts
        .into_iter()
        .map(|c| {
            serde_json::json!({
                "id": c.id,
                "username": c.username,
                "email": c.email,
                "created_at": c.created_at,
            })
        })
        .collect();

    Ok(Json(ApiResponse {
        success: true,
        data: Some(serialized_contacts),
        message: "Contacts retrieved".to_string(),
    }))
}

async fn add_contact(
    State(state): State<Arc<AppState>>,
    authed_user: AuthedUser,
    Json(payload): Json<AddContactPayload>,
) -> Result<Json<ApiResponse<String>>, QSafeError> {
    state
        .db
        .add_contact(&authed_user.id, &payload.contact_id)
        .await?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some("Contact added".to_string()),
        message: "Contact added successfully".to_string(),
    }))
}
