use std::net::TcpStream;

use super::protocol::{
    write_packet,
    PacketType,
};

pub fn send_file_offer(
    ip: String,
    filename: String,
    filesize: u64,
) {
    match TcpStream::connect(
        format!("{}:7878", ip)
    ) {

        Ok(mut stream) => {

            let payload =
                format!(
                    "{}|{}",
                    filename,
                    filesize
                );

            let _ = write_packet(
                &mut stream,
                PacketType::FileOffer,
                payload.as_bytes(),
            );
        }

        Err(e) => {
            println!("{}", e);
        }
    }
}