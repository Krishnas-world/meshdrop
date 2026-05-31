use std::fs::File;
use std::io::Read;
use std::net::TcpStream;
use std::path::Path;
use tauri::{AppHandle, Emitter};

use uuid::Uuid;

use super::protocol::{write_packet, PacketType};

use super::session::{sessions, TransferSession};

const CHUNK_SIZE: usize = 512 * 1024;
const CHUNK_MAGIC: &[u8; 4] = b"MDC1";

pub fn send_file_offer(ip: String, filename: String, filesize: u64) -> std::io::Result<String> {
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

    let mut stream = TcpStream::connect(format!("{}:7878", ip))?;
    let payload = format!("{}|{}|{}", transfer_id, filename, filesize);

    write_packet(&mut stream, PacketType::FileOffer, payload.as_bytes())?;

    println!("Offer sent: {}", payload);

    Ok(transfer_id)
}

pub fn send_file_data(
    ip: String,
    transfer_id: String,
    filename: String,
    bytes: Vec<u8>,
) -> std::io::Result<()> {
    let filename_bytes = filename.as_bytes();
    let transfer_id_bytes = transfer_id.as_bytes();

    if filename_bytes.len() > u16::MAX as usize {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Filename is too long",
        ));
    }

    if transfer_id_bytes.len() > u16::MAX as usize {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Transfer ID is too long",
        ));
    }

    let mut payload = Vec::with_capacity(
        4 + 2 + transfer_id_bytes.len() + 2 + filename_bytes.len() + bytes.len(),
    );
    payload.extend_from_slice(CHUNK_MAGIC);
    payload.extend_from_slice(&(transfer_id_bytes.len() as u16).to_be_bytes());
    payload.extend_from_slice(transfer_id_bytes);
    payload.extend_from_slice(&(filename_bytes.len() as u16).to_be_bytes());
    payload.extend_from_slice(filename_bytes);
    payload.extend_from_slice(&(bytes.len() as u64).to_be_bytes());
    payload.extend_from_slice(&0_u64.to_be_bytes());
    payload.extend_from_slice(&bytes);

    let mut stream = TcpStream::connect(format!("{}:7878", ip))?;
    write_packet(&mut stream, PacketType::FileData, &payload)?;

    println!("File data sent: {} ({} bytes)", filename, bytes.len());

    Ok(())
}

pub fn send_file_from_path(
    app: AppHandle,
    ip: String,
    transfer_id: String,
    path: String,
) -> std::io::Result<()> {
    let path = Path::new(&path);
    let filename = path
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid file path"))?
        .to_string();

    let filename_bytes = filename.as_bytes();
    let transfer_id_bytes = transfer_id.as_bytes();

    if filename_bytes.len() > u16::MAX as usize {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Filename is too long",
        ));
    }

    if transfer_id_bytes.len() > u16::MAX as usize {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Transfer ID is too long",
        ));
    }

    let mut file = File::open(path)?;
    let total_size = file.metadata()?.len();
    let mut offset = 0_u64;
    let mut buffer = vec![0_u8; CHUNK_SIZE];
    let mut stream = TcpStream::connect(format!("{}:7878", ip))?;

    loop {
        let read_count = file.read(&mut buffer)?;

        if read_count == 0 {
            break;
        }

        let mut payload = Vec::with_capacity(
            4 + 2 + transfer_id_bytes.len() + 2 + filename_bytes.len() + 8 + 8 + read_count,
        );
        payload.extend_from_slice(CHUNK_MAGIC);
        payload.extend_from_slice(&(transfer_id_bytes.len() as u16).to_be_bytes());
        payload.extend_from_slice(transfer_id_bytes);
        payload.extend_from_slice(&(filename_bytes.len() as u16).to_be_bytes());
        payload.extend_from_slice(filename_bytes);
        payload.extend_from_slice(&total_size.to_be_bytes());
        payload.extend_from_slice(&offset.to_be_bytes());
        payload.extend_from_slice(&buffer[..read_count]);

        write_packet(&mut stream, PacketType::FileData, &payload)?;

        offset += read_count as u64;
        let percent = if total_size == 0 {
            100
        } else {
            ((offset as f64 / total_size as f64) * 100.0).round() as u8
        };

        let _ = app.emit(
            "file-send-progress",
            format!(
                "{}|{}|{}|{}|{}",
                transfer_id, filename, offset, total_size, percent
            ),
        );
    }

    println!("File sent from path: {} ({} bytes)", filename, total_size);

    Ok(())
}
