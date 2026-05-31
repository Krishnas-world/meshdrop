use std::net::TcpStream;

use super::protocol::{write_packet, PacketType};

pub fn send_message(ip: String, message: String) {
    match TcpStream::connect(format!("{}:7878", ip)) {
        Ok(mut stream) => {
            match write_packet(&mut stream, PacketType::Message, message.as_bytes()) {
                Ok(_) => {
                    println!("Message sent");
                }

                Err(e) => {
                    println!("Failed to send packet: {}", e);
                }
            }
        }

        Err(e) => {
            println!("Connection failed: {}", e);
        }
    }
}
