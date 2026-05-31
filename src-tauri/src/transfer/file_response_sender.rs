use std::net::TcpStream;

use super::protocol::{write_packet, PacketType};

pub fn send_file_accept(ip: String) {
    match TcpStream::connect(format!("{}:7878", ip)) {
        Ok(mut stream) => {
            let _ = write_packet(&mut stream, PacketType::FileAccept, b"accepted");

            println!("Accept sent");
        }

        Err(e) => {
            println!("{}", e);
        }
    }
}

pub fn send_file_reject(ip: String) {
    match TcpStream::connect(format!("{}:7878", ip)) {
        Ok(mut stream) => {
            let _ = write_packet(&mut stream, PacketType::FileReject, b"rejected");

            println!("Reject sent");
        }

        Err(e) => {
            println!("{}", e);
        }
    }
}
