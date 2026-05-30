//! HSM Communication Interfaces and Mock Simulator for host-server

use crate::error::QSafeError;
use pqcrypto_kyber::kyber768::{decapsulate, keypair};
use pqcrypto_traits::kem::{Ciphertext, PublicKey, SecretKey, SharedSecret};
use qsafe_common::PacketType;
use rand::RngCore;
use std::io::{Read, Write};
use std::time::Duration;

pub trait HsmConnection: Send {
    fn send_request(
        &mut self,
        packet_type: PacketType,
        payload: &[u8],
    ) -> Result<Vec<u8>, QSafeError>;
}

/// In-memory local HSM simulator for offline HIL testing
pub struct MockHsmConnection {
    public_key: Vec<u8>,
    secret_key: Vec<u8>,
}

impl MockHsmConnection {
    pub fn new() -> Self {
        let (pk, sk) = keypair();
        Self {
            public_key: pk.as_bytes().to_vec(),
            secret_key: sk.as_bytes().to_vec(),
        }
    }
}

impl Default for MockHsmConnection {
    fn default() -> Self {
        Self::new()
    }
}

impl HsmConnection for MockHsmConnection {
    fn send_request(
        &mut self,
        packet_type: PacketType,
        payload: &[u8],
    ) -> Result<Vec<u8>, QSafeError> {
        match packet_type {
            PacketType::GetPublicKeyReq => Ok(self.public_key.clone()),
            PacketType::RandomBytesReq => {
                if payload.len() != 2 {
                    return Err(QSafeError::BadRequest(
                        "Payload must be 2 bytes for length".to_string(),
                    ));
                }
                let len = u16::from_be_bytes([payload[0], payload[1]]) as usize;
                let mut data = vec![0u8; len];
                rand::thread_rng().fill_bytes(&mut data);
                Ok(data)
            }
            PacketType::KyberDecapsulateReq => {
                let sk = SecretKey::from_bytes(&self.secret_key)
                    .map_err(|_| QSafeError::Crypto("Invalid mock secret key".to_string()))?;
                let ct = Ciphertext::from_bytes(payload)
                    .map_err(|_| QSafeError::Crypto("Invalid mock ciphertext".to_string()))?;
                let ss = decapsulate(&ct, &sk);
                Ok(ss.as_bytes().to_vec())
            }
            _ => Err(QSafeError::BadRequest(
                "Unsupported mock packet type".to_string(),
            )),
        }
    }
}

/// Physical USB-CDC Serial Port HSM connection driver
pub struct PhysicalHsmConnection {
    port: Box<dyn serialport::SerialPort>,
}

impl PhysicalHsmConnection {
    pub fn new(port_name: &str) -> Result<Self, QSafeError> {
        let port = serialport::new(port_name, 115_200)
            .timeout(Duration::from_secs(2))
            .open()
            .map_err(|e| {
                QSafeError::Internal(format!("Failed to open serial port {}: {}", port_name, e))
            })?;
        Ok(Self { port })
    }
}

impl HsmConnection for PhysicalHsmConnection {
    fn send_request(
        &mut self,
        packet_type: PacketType,
        payload: &[u8],
    ) -> Result<Vec<u8>, QSafeError> {
        let mut tx_buf =
            vec![0u8; qsafe_common::HEADER_LEN + payload.len() + qsafe_common::CRC_LEN];
        let total_tx = qsafe_common::encode_packet(packet_type, payload, &mut tx_buf)
            .map_err(|e| QSafeError::Internal(format!("Packet encode failed: {}", e)))?;

        self.port
            .clear(serialport::ClearBuffer::Input)
            .map_err(|e| QSafeError::Internal(format!("Failed to clear serial input: {}", e)))?;

        self.port
            .write_all(&tx_buf[..total_tx])
            .map_err(|e| QSafeError::Crypto(format!("Failed to write serial packet: {}", e)))?;

        let mut header = [0u8; qsafe_common::HEADER_LEN];
        self.port
            .read_exact(&mut header)
            .map_err(|e| QSafeError::Crypto(format!("Failed to read packet header: {}", e)))?;

        let payload_len = u16::from_be_bytes([header[1], header[2]]) as usize;

        let mut body = vec![0u8; payload_len + qsafe_common::CRC_LEN];
        self.port
            .read_exact(&mut body)
            .map_err(|e| QSafeError::Crypto(format!("Failed to read packet body: {}", e)))?;

        let mut rx_buf =
            Vec::with_capacity(qsafe_common::HEADER_LEN + payload_len + qsafe_common::CRC_LEN);
        rx_buf.extend_from_slice(&header);
        rx_buf.extend_from_slice(&body);

        let decoded = qsafe_common::decode_packet(&rx_buf)
            .map_err(|e| QSafeError::Crypto(format!("Packet decode validation failed: {}", e)))?;

        let expected_resp = match packet_type {
            PacketType::GetPublicKeyReq => PacketType::GetPublicKeyResp,
            PacketType::RandomBytesReq => PacketType::RandomBytesResp,
            PacketType::KyberDecapsulateReq => PacketType::KyberDecapsulateResp,
            _ => PacketType::Unknown,
        };

        if decoded.packet_type != expected_resp {
            return Err(QSafeError::Crypto(format!(
                "Unexpected packet response type: {:?}",
                decoded.packet_type
            )));
        }

        Ok(decoded.payload.to_vec())
    }
}
