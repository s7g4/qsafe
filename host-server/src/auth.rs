//! Authentication module for Q-Safe

use crate::error::QSafeError;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user id
    pub username: String,
    pub exp: usize,
    pub token_type: String, // "access" or "refresh"
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub user_id: String,
    pub username: String,
}

pub struct AuthService {
    jwt_secret: String,
}

impl AuthService {
    pub fn new(jwt_secret: String) -> Self {
        Self { jwt_secret }
    }

    pub fn hash_password(&self, password: &str) -> Result<String, QSafeError> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let hash = argon2
            .hash_password(password.as_bytes(), &salt)?
            .to_string();
        Ok(hash)
    }

    pub fn verify_password(&self, password: &str, hash_str: &str) -> Result<bool, QSafeError> {
        let parsed_hash = PasswordHash::new(hash_str)?;
        let result = Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok();
        Ok(result)
    }

    pub fn create_token(&self, user_id: &Uuid, username: &str) -> Result<String, QSafeError> {
        self.create_access_token(user_id, username)
    }

    pub fn create_access_token(
        &self,
        user_id: &Uuid,
        username: &str,
    ) -> Result<String, QSafeError> {
        let expiration = Utc::now()
            .checked_add_signed(Duration::minutes(15))
            .ok_or_else(|| {
                QSafeError::Internal("Failed to calculate access token expiration".to_string())
            })?
            .timestamp() as usize;

        let claims = Claims {
            sub: user_id.to_string(),
            username: username.to_string(),
            exp: expiration,
            token_type: "access".to_string(),
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_ref()),
        )?;
        Ok(token)
    }

    pub fn create_refresh_token(
        &self,
        user_id: &Uuid,
        username: &str,
    ) -> Result<String, QSafeError> {
        let expiration = Utc::now()
            .checked_add_signed(Duration::days(7))
            .ok_or_else(|| {
                QSafeError::Internal("Failed to calculate refresh token expiration".to_string())
            })?
            .timestamp() as usize;

        let claims = Claims {
            sub: user_id.to_string(),
            username: username.to_string(),
            exp: expiration,
            token_type: "refresh".to_string(),
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_ref()),
        )?;
        Ok(token)
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims, QSafeError> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_ref()),
            &Validation::default(),
        )?;

        Ok(token_data.claims)
    }

    pub fn extract_user_id_from_token(&self, token: &str) -> Result<Uuid, QSafeError> {
        let claims = self.verify_token(token)?;
        if claims.token_type != "access" {
            return Err(QSafeError::Unauthorized("Invalid token type".to_string()));
        }
        let id = Uuid::parse_str(&claims.sub)
            .map_err(|_| QSafeError::Unauthorized("Invalid user ID in token".to_string()))?;
        Ok(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_password_hashing() {
        let auth = AuthService::new("test_secret".to_string());
        let password = "my_super_secret_password";
        
        let hash = auth.hash_password(password).expect("Hashing failed");
        assert!(auth.verify_password(password, &hash).unwrap());
        assert!(!auth.verify_password("wrong_password", &hash).unwrap());
    }

    #[test]
    fn test_jwt_creation_and_validation() {
        let auth = AuthService::new("test_secret".to_string());
        let user_id = Uuid::new_v4();
        let username = "testuser";

        let access_token = auth.create_access_token(&user_id, username).unwrap();
        let claims = auth.verify_token(&access_token).unwrap();
        
        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.username, username);
        assert_eq!(claims.token_type, "access");
    }

    #[test]
    fn test_extract_user_id() {
        let auth = AuthService::new("test_secret".to_string());
        let user_id = Uuid::new_v4();
        
        let access_token = auth.create_access_token(&user_id, "user").unwrap();
        let extracted_id = auth.extract_user_id_from_token(&access_token).unwrap();
        
        assert_eq!(user_id, extracted_id);
    }
}
