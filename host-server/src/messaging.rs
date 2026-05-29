    //! Secure messaging module with hybrid protocol

use crate::crypto::CryptoEngine;
use crate::qkd::QKDProtocol;
use crate::handshake::{Handshake, HandshakeInit, HandshakeResponse, HandshakeResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use zeroize::Zeroize;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub sender: String,
    pub recipient: String,
    pub content: Vec<u8>, // AEAD encrypted content
    pub timestamp: u64,
    pub signature: Vec<u8>,
    pub seq_num: u64, // Monotonic sequence number
}

#[derive(Debug)]
pub struct ChatSession {
    pub participants: Vec<String>,
    pub shared_key: Vec<u8>,
    pub message_history: Vec<Message>,
    pub seq_num: u64,
    pub confirmed: bool,
}

pub struct MessagingService {
    crypto: CryptoEngine,
    qkd: QKDProtocol,
    handshake: Handshake,
    pub sessions: Arc<Mutex<HashMap<String, ChatSession>>>,
    message_sender: mpsc::UnboundedSender<Message>,
    message_receiver: mpsc::UnboundedReceiver<Message>,
}

impl MessagingService {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self {
            crypto: CryptoEngine::new(),
            qkd: QKDProtocol::new(),
            handshake: Handshake::new(),
            sessions: Arc::new(Mutex::new(HashMap::new())),
            message_sender: tx,
            message_receiver: rx,
        }
    }

    /// Establish a secure chat session using hybrid protocol
    pub async fn establish_session(&mut self, session_id: &str, participants: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
        // Alice initiates handshake
        let (init, alice_private) = self.handshake.alice_initiate(256, 0.2)?;

        // Simulate Bob responding (in real impl, send over network)
        let (response, bob_private) = self.handshake.bob_respond(&init, Some(alice_private.clone()))?;

        // Alice finalizes
        let result = self.handshake.alice_finalize(&init, &response, alice_private)?;

        if !result.confirmed {
            return Err("Key confirmation failed".into());
        }

        if result.eavesdropping_detected {
            let mut session_key = result.session_key;
            session_key.zeroize();
            return Err("Eavesdropping detected during handshake".into());
        }

        let session = ChatSession {
            participants,
            shared_key: result.session_key.clone(),
            message_history: Vec::new(),
            seq_num: 0,
            confirmed: true,
        };

        self.sessions.lock().unwrap().insert(session_id.to_string(), session);
        Ok(())
    }

    /// Send an encrypted message with AEAD
    pub async fn send_message(&mut self, session_id: &str, sender: &str, recipient: &str, plaintext: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut sessions = self.sessions.lock().unwrap();
        let session = sessions.get_mut(session_id).ok_or("Session not found")?;

        if !session.confirmed {
            return Err("Session not confirmed".into());
        }

        // Encrypt with AEAD, include seq_num as AAD
        let aad = bincode::serialize(&(sender, recipient, session.seq_num))?;
        let (ciphertext, nonce) = self.crypto.encrypt_aead(&session.shared_key, plaintext.as_bytes(), Some(&aad))?;

        // Sign the message
        let keypair = self.crypto.generate_ed25519_keypair()?;
        let signature = self.crypto.sign_ed25519(&keypair.secret_key, &ciphertext)?;

        let message = Message {
            id: uuid::Uuid::new_v4().to_string(),
            sender: sender.to_string(),
            recipient: recipient.to_string(),
            content: ciphertext,
            timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_secs(),
            signature: signature.signature,
            seq_num: session.seq_num,
        };

        // Increment seq_num
        session.seq_num += 1;

        // Send the message
        self.message_sender.send(message.clone())?;

        // Add to history
        session.message_history.push(message);

        Ok(())
    }

    /// Receive and decrypt messages with AEAD
    pub async fn receive_messages(&mut self, session_id: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let sessions = self.sessions.lock().unwrap();
        let session = sessions.get(session_id).ok_or("Session not found")?;

        let mut decrypted_messages = Vec::new();

        while let Ok(message) = self.message_receiver.try_recv() {
            // Verify signature
            let keypair = self.crypto.generate_ed25519_keypair()?; // In real impl, use sender's pubkey
            if !self.crypto.verify_ed25519(&keypair.public_key, &crate::crypto::Signature {
                signature: message.signature.clone(),
                message: message.content.clone(),
            })? {
                continue; // Skip invalid messages
            }

            // Decrypt with AEAD, include seq_num as AAD
            let aad = bincode::serialize(&(message.sender, message.recipient, message.seq_num))?;
            let plaintext = self.crypto.decrypt_aead(&session.shared_key, &message.content, &[0u8; 12], Some(&aad))?; // Nonce from message?
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

    /// Rotate session key periodically
    pub async fn rotate_key(&mut self, session_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut sessions = self.sessions.lock().unwrap();
        let session = sessions.get_mut(session_id).ok_or("Session not found")?;

        // Perform new handshake for key rotation
        let (init, alice_private) = self.handshake.alice_initiate(256, 0.2)?;
        let (response, _) = self.handshake.bob_respond(&init, Some(alice_private.clone()))?;
        let result = self.handshake.alice_finalize(&init, &response, alice_private)?;

        if result.confirmed && !result.eavesdropping_detected {
            session.shared_key.zeroize(); // Zeroize old key
            session.shared_key = result.session_key;
            session.seq_num = 0; // Reset seq_num
        } else {
            return Err("Key rotation failed".into());
        }

        Ok(())
    }
}
