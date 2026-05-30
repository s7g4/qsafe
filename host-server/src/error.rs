//! Custom Error type for Q-Safe

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum QSafeError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Cryptographic error: {0}")]
    Crypto(String),

    #[error("Password hashing error: {0}")]
    Argon2(String),

    #[error("Token error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),

    #[error("User conflict error: {0}")]
    UserConflict(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Internal server error: {0}")]
    Internal(String),
}

impl From<argon2::password_hash::Error> for QSafeError {
    fn from(err: argon2::password_hash::Error) -> Self {
        QSafeError::Argon2(err.to_string())
    }
}

impl IntoResponse for QSafeError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            QSafeError::Database(ref e) => {
                // Generified error to prevent database schema exposure to client
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Database error: {}", e),
                )
            }
            QSafeError::Crypto(ref msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            QSafeError::Argon2(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to process authentication hash".to_string(),
            ),
            QSafeError::Jwt(ref e) => (
                StatusCode::UNAUTHORIZED,
                format!("Token validation failed: {}", e),
            ),
            QSafeError::UserConflict(ref msg) => (StatusCode::CONFLICT, msg.clone()),
            QSafeError::Unauthorized(ref msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
            QSafeError::NotFound(ref msg) => (StatusCode::NOT_FOUND, msg.clone()),
            QSafeError::BadRequest(ref msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            QSafeError::Internal(ref msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
        };

        let body = Json(json!({
            "success": false,
            "data": null,
            "message": message
        }));

        (status, body).into_response()
    }
}
