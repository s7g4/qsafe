//! Shared definitions and packet framing for the Q-Safe host and HSM.
#![no_std]

pub const HEADER_LEN: usize = 3; // Type (1B) + Length (2B)
pub const CRC_LEN: usize = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacketType {
    RandomBytesReq = 0x01,
    RandomBytesResp = 0x02,
    KyberDecapsulateReq = 0x03,
    KyberDecapsulateResp = 0x04,
    GetPublicKeyReq = 0x05,
    GetPublicKeyResp = 0x06,
    Unknown = 0xFF,
}

impl From<u8> for PacketType {
    fn from(val: u8) -> Self {
        match val {
            0x01 => PacketType::RandomBytesReq,
            0x02 => PacketType::RandomBytesResp,
            0x03 => PacketType::KyberDecapsulateReq,
            0x04 => PacketType::KyberDecapsulateResp,
            0x05 => PacketType::GetPublicKeyReq,
            0x06 => PacketType::GetPublicKeyResp,
            _ => PacketType::Unknown,
        }
    }
}

impl From<PacketType> for u8 {
    fn from(val: PacketType) -> u8 {
        val as u8
    }
}

/// Compute standard CRC-16-CCITT checksum (Poly: 0x1021, Init: 0xFFFF)
pub fn crc16_ccitt(data: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;
    for &byte in data {
        crc ^= (byte as u16) << 8;
        for _ in 0..8 {
            if (crc & 0x8000) != 0 {
                crc = (crc << 1) ^ 0x1021;
            } else {
                crc <<= 1;
            }
        }
    }
    crc
}

/// Encode a packet into the target byte buffer. Returns the total byte length encoded.
pub fn encode_packet(
    packet_type: PacketType,
    payload: &[u8],
    buf: &mut [u8],
) -> Result<usize, &'static str> {
    let payload_len = payload.len();
    if payload_len > 0xFFFF {
        return Err("Payload too large");
    }
    let total_len = HEADER_LEN + payload_len + CRC_LEN;
    if buf.len() < total_len {
        return Err("Buffer too small");
    }

    buf[0] = packet_type.into();

    let len_bytes = (payload_len as u16).to_be_bytes();
    buf[1..3].copy_from_slice(&len_bytes);

    buf[3..3 + payload_len].copy_from_slice(payload);

    let crc = crc16_ccitt(&buf[0..3 + payload_len]);
    let crc_bytes = crc.to_be_bytes();

    buf[3 + payload_len..5 + payload_len].copy_from_slice(&crc_bytes);

    Ok(total_len)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedPacket<'a> {
    pub packet_type: PacketType,
    pub payload: &'a [u8],
}

/// Decode a packet from a byte slice. Performs length and CRC checks.
pub fn decode_packet(buf: &[u8]) -> Result<DecodedPacket<'_>, &'static str> {
    if buf.len() < HEADER_LEN + CRC_LEN {
        return Err("Buffer too short");
    }

    let packet_type = PacketType::from(buf[0]);
    if packet_type == PacketType::Unknown {
        return Err("Unknown packet type");
    }

    let payload_len = u16::from_be_bytes([buf[1], buf[2]]) as usize;
    let expected_total_len = HEADER_LEN + payload_len + CRC_LEN;
    if buf.len() < expected_total_len {
        return Err("Buffer does not contain full packet");
    }

    let rx_crc = u16::from_be_bytes([buf[3 + payload_len], buf[4 + payload_len]]);
    let calc_crc = crc16_ccitt(&buf[0..3 + payload_len]);
    if rx_crc != calc_crc {
        return Err("CRC mismatch");
    }

    Ok(DecodedPacket {
        packet_type,
        payload: &buf[3..3 + payload_len],
    })
}
