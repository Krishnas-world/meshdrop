use std::io::{Read, Write};
use std::net::TcpStream;

pub const PROTOCOL_VERSION: u8 = 1;

#[derive(Debug, Clone, Copy)]
pub enum PacketType {
    Message = 1,
    FileOffer = 2,
    FileAccept = 3,
    FileReject = 4,
    FileData = 5,
}

impl PacketType {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            1 => Some(PacketType::Message),
            2 => Some(PacketType::FileOffer),
            3 => Some(PacketType::FileAccept),
            4 => Some(PacketType::FileReject),
            5 => Some(PacketType::FileData),
            _ => None,
        }
    }
}

pub struct Packet {
    pub packet_type: PacketType,
    pub payload: Vec<u8>,
}

pub fn write_packet(
    stream: &mut TcpStream,
    packet_type: PacketType,
    payload: &[u8],
) -> std::io::Result<()> {
    // Version
    stream.write_all(&[PROTOCOL_VERSION])?;

    // Type
    stream.write_all(&[packet_type as u8])?;

    // Payload Length (u64)
    let payload_length = payload.len() as u64;
    stream.write_all(&payload_length.to_be_bytes())?;

    // Payload
    stream.write_all(payload)?;

    Ok(())
}

pub fn read_packet(stream: &mut TcpStream) -> std::io::Result<Packet> {
    let mut version_buf = [0u8; 1];
    stream.read_exact(&mut version_buf)?;

    if version_buf[0] != PROTOCOL_VERSION {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Unsupported protocol version",
        ));
    }

    let mut type_buf = [0u8; 1];
    stream.read_exact(&mut type_buf)?;

    let packet_type = PacketType::from_u8(type_buf[0]).ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, "Unknown packet type")
    })?;

    let mut length_buf = [0u8; 8];
    stream.read_exact(&mut length_buf)?;

    let payload_length = u64::from_be_bytes(length_buf);

    let mut payload = vec![0u8; payload_length as usize];

    stream.read_exact(&mut payload)?;

    Ok(Packet {
        packet_type,
        payload,
    })
}
