use std::net::TcpStream;

use super::protocol::{write_packet, PacketType};

pub fn send_file_accept(ip: String, transfer_id: String) {
    match TcpStream::connect(format!("{}:7878", ip)) {
        Ok(mut stream) => {
            let _ = write_packet(&mut stream, PacketType::FileAccept, transfer_id.as_bytes());

            println!("Accept sent: {}", transfer_id);
        }

        Err(e) => {
            println!("{}", e);
        }
    }
}

pub fn send_file_reject(ip: String, transfer_id: String) {
    match TcpStream::connect(format!("{}:7878", ip)) {
        Ok(mut stream) => {
            let _ = write_packet(&mut stream, PacketType::FileReject, transfer_id.as_bytes());

            println!("Reject sent: {}", transfer_id);
        }

        Err(e) => {
            println!("{}", e);
        }
    }
}
