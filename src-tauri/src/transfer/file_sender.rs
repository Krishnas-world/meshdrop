use std::net::TcpStream;

use uuid::Uuid;

use super::protocol::{write_packet, PacketType};

use super::session::{sessions, TransferSession};

pub fn send_file_offer(ip: String, filename: String, filesize: u64) {
    let transfer_id = Uuid::new_v4().to_string();

    let session = TransferSession {
        id: transfer_id.clone(),
        file_name: filename.clone(),
        file_size: filesize,
        file_path: String::new(),
    };

    sessions()
        .lock()
        .unwrap()
        .insert(transfer_id.clone(), session);

    match TcpStream::connect(format!("{}:7878", ip)) {
        Ok(mut stream) => {
            let payload = format!("{}|{}|{}", transfer_id, filename, filesize);

            let _ = write_packet(&mut stream, PacketType::FileOffer, payload.as_bytes());

            println!("Offer sent: {}", payload);
        }

        Err(e) => {
            println!("{}", e);
        }
    }
}

pub fn send_file_data(ip: String, filename: String, bytes: Vec<u8>) -> std::io::Result<()> {
    let filename_bytes = filename.as_bytes();

    if filename_bytes.len() > u16::MAX as usize {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Filename is too long",
        ));
    }

    let mut payload = Vec::with_capacity(2 + filename_bytes.len() + bytes.len());
    payload.extend_from_slice(&(filename_bytes.len() as u16).to_be_bytes());
    payload.extend_from_slice(filename_bytes);
    payload.extend_from_slice(&bytes);

    let mut stream = TcpStream::connect(format!("{}:7878", ip))?;
    write_packet(&mut stream, PacketType::FileData, &payload)?;

    println!("File data sent: {} ({} bytes)", filename, bytes.len());

    Ok(())
}
