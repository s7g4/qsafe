//! Secure messaging module

use crate::crypto::CryptoEngine;
use crate::qkd::QKDProtocol;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub sender: String,
    pub recipient: String,
    pub content: Vec<u8>, // Encrypted content
    pub timestamp: u64,
    pub signature: Vec<u8>,
}

#[derive(Debug)]
pub struct ChatSession {
    pub participants: Vec<String>,
    pub shared_key: Vec<u8>,
    pub message_history: Vec<Message>,
}

pub struct MessagingService {
    crypto: CryptoEngine,
    qkd: QKDProtocol,
    pub sessions: Arc<Mutex<HashMap<String, ChatSession>>>,
    message_sender: mpsc::UnboundedSender<Message>,
    message_receiver: mpsc::UnboundedReceiver<Message>,
}

impl MessagingService {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self {
            crypto: CryptoEngine::new(),
            qkd: QKDProtocol::new(42), // Fixed seed for demo
            sessions: Arc::new(Mutex::new(HashMap::new())),
            message_sender: tx,
            message_receiver: rx,
        }
    }

    /// Establish a secure chat session using QKD
    pub async fn establish_session(&mut self, session_id: &str, participants: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
        // Perform QKD key exchange
        let qkd_key = self.qkd.bb84_key_exchange(256)?;

        if qkd_key.eavesdropping_detected {
            return Err("Eavesdropping detected during key exchange".into());
        }

        let session = ChatSession {
            participants,
            shared_key: qkd_key.key,
            message_history: Vec::new(),
        };

        self.sessions.lock().unwrap().insert(session_id.to_string(), session);
        Ok(())
    }

    /// Send an encrypted message
    pub async fn send_message(&mut self, session_id: &str, sender: &str, recipient: &str, plaintext: &str) -> Result<(), Box<dyn std::error::Error>> {
        let sessions = self.sessions.lock().unwrap();
        let session = sessions.get(session_id).ok_or("Session not found")?;

        // Encrypt the message
        let (mut ciphertext, nonce, _) = self.crypto.encrypt(&session.shared_key, plaintext.as_bytes())?;
        ciphertext.extend_from_slice(&nonce);

        // Sign the message
        let keypair = self.crypto.generate_pq_keypair()?;
        let signature = self.crypto.sign(&keypair.secret_key, &ciphertext)?;

        let message = Message {
            id: uuid::Uuid::new_v4().to_string(),
            sender: sender.to_string(),
            recipient: recipient.to_string(),
            content: ciphertext,
            timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_secs(),
            signature: signature.signature,
        };

        // Send the message
        self.message_sender.send(message.clone())?;

        // Add to history
        drop(sessions);
        let mut sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get_mut(session_id) {
            session.message_history.push(message);
        }

        Ok(())
    }

    /// Receive and decrypt messages
    pub async fn receive_messages(&mut self, session_id: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let sessions = self.sessions.lock().unwrap();
        let session = sessions.get(session_id).ok_or("Session not found")?;

        let mut decrypted_messages = Vec::new();

        while let Ok(message) = self.message_receiver.try_recv() {
            // Verify signature (simplified - in real implementation, use sender's public key)
            // For demo, we'll skip signature verification

            // Decrypt the message
            let nonce = &message.content[message.content.len()-12..]; // Last 12 bytes are nonce in our implementation
            let ciphertext = &message.content[..message.content.len()-12];

            let plaintext = self.crypto.decrypt(&session.shared_key, ciphertext, nonce)?;
            let text = String::from_utf8(plaintext)?;

            decrypted_messages.push(text);
        }

        Ok(decrypted_messages)
    }

    /// Get message history for a session
    pub fn get_message_history(&self, session_id: &str) -> Result<Vec<Message>, Box<dyn std::error::Error>> {
        let sessions = self.sessions.lock().unwrap();
        let session = sessions.get(session_id).ok_or("Session not found")?;
        Ok(session.message_history.clone())
    }
}
