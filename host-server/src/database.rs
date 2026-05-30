//! Database module for Q-Safe

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, PgPool};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub public_key: Vec<u8>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Message {
    pub id: Uuid,
    pub sender_id: Uuid,
    pub recipient_id: Uuid,
    pub encrypted_content: Vec<u8>,
    pub nonce: Vec<u8>,
    pub timestamp: DateTime<Utc>,
    pub session_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct ChatSession {
    pub id: Uuid,
    pub participants: Vec<Uuid>,
    pub shared_key: Vec<u8>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Contact {
    pub user_id: Uuid,
    pub contact_id: Uuid,
    pub status: String, // "pending", "accepted", "blocked"
}

#[derive(Clone)]
pub struct Database {
    pool: PgPool,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct OfflineMessage {
    pub id: Uuid,
    pub recipient_id: Uuid,
    pub sender_id: Uuid,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;

        // Run database migrations automatically
        sqlx::migrate!().run(&pool).await?;

        Ok(Self { pool })
    }

    // User operations
    pub async fn create_user(
        &self,
        username: &str,
        email: &str,
        password_hash: &str,
        public_key: &[u8],
    ) -> Result<User, sqlx::Error> {
        let user = sqlx::query_as::<_, User>(
            "INSERT INTO users (username, email, password_hash, public_key) VALUES ($1, $2, $3, $4) RETURNING *",
        )
        .bind(username)
        .bind(email)
        .bind(password_hash)
        .bind(public_key)
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn get_user_by_username(&self, username: &str) -> Result<Option<User>, sqlx::Error> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = $1")
            .bind(username)
            .fetch_optional(&self.pool)
            .await?;

        Ok(user)
    }

    pub async fn get_user_by_id(&self, id: &Uuid) -> Result<Option<User>, sqlx::Error> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(user)
    }

    // Session operations
    pub async fn create_session(
        &self,
        participants: &[Uuid],
        shared_key: &[u8],
    ) -> Result<ChatSession, sqlx::Error> {
        let session = sqlx::query_as::<_, ChatSession>(
            "INSERT INTO chat_sessions (participants, shared_key) VALUES ($1, $2) RETURNING *",
        )
        .bind(participants)
        .bind(shared_key)
        .fetch_one(&self.pool)
        .await?;

        Ok(session)
    }

    pub async fn get_session(&self, session_id: &Uuid) -> Result<Option<ChatSession>, sqlx::Error> {
        let session = sqlx::query_as::<_, ChatSession>("SELECT * FROM chat_sessions WHERE id = $1")
            .bind(session_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(session)
    }

    // Message operations
    pub async fn save_message(
        &self,
        sender_id: &Uuid,
        recipient_id: &Uuid,
        encrypted_content: &[u8],
        nonce: &[u8],
        session_id: &Uuid,
    ) -> Result<Message, sqlx::Error> {
        let message = sqlx::query_as::<_, Message>(
            "INSERT INTO messages (sender_id, recipient_id, encrypted_content, nonce, session_id) VALUES ($1, $2, $3, $4, $5) RETURNING *",
        )
        .bind(sender_id)
        .bind(recipient_id)
        .bind(encrypted_content)
        .bind(nonce)
        .bind(session_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(message)
    }

    pub async fn get_messages_between_users(
        &self,
        user1: &Uuid,
        user2: &Uuid,
        limit: i64,
    ) -> Result<Vec<Message>, sqlx::Error> {
        let messages = sqlx::query_as::<_, Message>(
            "SELECT * FROM messages WHERE (sender_id = $1 AND recipient_id = $2) OR (sender_id = $2 AND recipient_id = $1) ORDER BY timestamp DESC LIMIT $3",
        )
        .bind(user1)
        .bind(user2)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(messages)
    }

    // Contact operations
    pub async fn add_contact(&self, user_id: &Uuid, contact_id: &Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO contacts (user_id, contact_id, status) VALUES ($1, $2, 'accepted') ON CONFLICT (user_id, contact_id) DO NOTHING",
        )
        .bind(user_id)
        .bind(contact_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_contacts(&self, user_id: &Uuid) -> Result<Vec<User>, sqlx::Error> {
        let contacts = sqlx::query_as::<_, User>(
            "SELECT u.* FROM users u INNER JOIN contacts c ON u.id = c.contact_id WHERE c.user_id = $1 AND c.status = 'accepted'",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(contacts)
    }

    // Offline message operations
    pub async fn save_offline_message(
        &self,
        recipient_id: &Uuid,
        sender_id: &Uuid,
        content: &str,
    ) -> Result<OfflineMessage, sqlx::Error> {
        let msg = sqlx::query_as::<_, OfflineMessage>(
            "INSERT INTO offline_messages (recipient_id, sender_id, content) VALUES ($1, $2, $3) RETURNING *",
        )
        .bind(recipient_id)
        .bind(sender_id)
        .bind(content)
        .fetch_one(&self.pool)
        .await?;

        Ok(msg)
    }

    pub async fn get_offline_messages(
        &self,
        recipient_id: &Uuid,
    ) -> Result<Vec<OfflineMessage>, sqlx::Error> {
        let msgs = sqlx::query_as::<_, OfflineMessage>(
            "SELECT * FROM offline_messages WHERE recipient_id = $1 ORDER BY created_at ASC",
        )
        .bind(recipient_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(msgs)
    }

    pub async fn clear_offline_messages(&self, recipient_id: &Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM offline_messages WHERE recipient_id = $1")
            .bind(recipient_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
