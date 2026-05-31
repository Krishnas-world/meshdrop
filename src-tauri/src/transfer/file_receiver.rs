use super::session::{sessions, TransferSession};
use std::fs::{self, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use tauri::{AppHandle, Emitter};

static RECEIVE_FOLDER: OnceLock<Mutex<PathBuf>> = OnceLock::new();
const CHUNK_MAGIC: &[u8; 4] = b"MDC1";

fn default_receive_folder() -> PathBuf {
    std::env::var_os("USERPROFILE")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(PathBuf::from))
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
        .join("Downloads")
        .join("MeshDrop")
}

fn receive_folder() -> &'static Mutex<PathBuf> {
    RECEIVE_FOLDER.get_or_init(|| Mutex::new(default_receive_folder()))
}

pub fn get_receive_folder() -> String {
    receive_folder()
        .lock()
        .unwrap()
        .to_string_lossy()
        .to_string()
}

pub fn set_receive_folder(path: String) -> std::io::Result<String> {
    let folder = PathBuf::from(path.trim());

    if folder.as_os_str().is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Receive folder cannot be empty",
        ));
    }

    fs::create_dir_all(&folder)?;
    *receive_folder().lock().unwrap() = folder.clone();

    Ok(folder.to_string_lossy().to_string())
}

pub fn set_receive_location(path: String) -> std::io::Result<String> {
    let location = PathBuf::from(path.trim());

    if location.as_os_str().is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Receive location cannot be empty",
        ));
    }

    set_receive_folder(location.join("MeshDrop").to_string_lossy().to_string())
}

pub fn handle_file_offer(app: &AppHandle, transfer_id: String, filename: String, filesize: u64) {
    sessions().lock().unwrap().insert(
        transfer_id.clone(),
        TransferSession {
            id: transfer_id.clone(),
            file_name: filename.clone(),
            file_size: filesize,
            file_path: String::new(),
        },
    );

    let payload = format!("{}|{}|{}", transfer_id, filename, filesize);

    let _ = app.emit("incoming-file-offer", payload);
}

pub fn handle_file_data(app: &AppHandle, payload: &[u8]) -> std::io::Result<PathBuf> {
    if payload.starts_with(CHUNK_MAGIC) {
        return handle_file_chunk(app, payload);
    }

    if payload.len() < 2 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "File data payload is missing filename length",
        ));
    }

    let filename_len = u16::from_be_bytes([payload[0], payload[1]]) as usize;
    let data_start = 2 + filename_len;

    if payload.len() < data_start {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "File data payload is missing filename",
        ));
    }

    let filename = String::from_utf8_lossy(&payload[2..data_start]).to_string();
    let safe_filename = Path::new(&filename)
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .unwrap_or("meshdrop-file");

    let folder = receive_folder().lock().unwrap().clone();
    fs::create_dir_all(&folder)?;

    let path = folder.join(safe_filename);
    fs::write(&path, &payload[data_start..])?;

    let _ = app.emit("file-received", path.to_string_lossy().to_string());

    Ok(path)
}

fn handle_file_chunk(app: &AppHandle, payload: &[u8]) -> std::io::Result<PathBuf> {
    if payload.len() < 4 + 2 + 2 + 8 + 8 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Chunk payload is too small",
        ));
    }

    let transfer_id_len = u16::from_be_bytes([payload[4], payload[5]]) as usize;
    let filename_len_start = 6 + transfer_id_len;

    if payload.len() < filename_len_start + 2 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Chunk payload is missing transfer ID",
        ));
    }

    let transfer_id = String::from_utf8_lossy(&payload[6..filename_len_start]).to_string();
    let filename_len =
        u16::from_be_bytes([payload[filename_len_start], payload[filename_len_start + 1]]) as usize;
    let filename_start = filename_len_start + 2;
    let total_start = filename_start + filename_len;
    let offset_start = total_start + 8;
    let data_start = offset_start + 8;

    if payload.len() < data_start {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Chunk payload is missing metadata",
        ));
    }

    let filename = String::from_utf8_lossy(&payload[filename_start..total_start]).to_string();
    let safe_filename = Path::new(&filename)
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .unwrap_or("meshdrop-file");
    let total_size = u64::from_be_bytes(
        payload[total_start..offset_start]
            .try_into()
            .expect("total size slice has exact length"),
    );
    let offset = u64::from_be_bytes(
        payload[offset_start..data_start]
            .try_into()
            .expect("offset slice has exact length"),
    );
    let chunk = &payload[data_start..];

    let folder = receive_folder().lock().unwrap().clone();
    fs::create_dir_all(&folder)?;

    let path = folder.join(safe_filename);
    let path_string = path.to_string_lossy().to_string();
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(offset == 0)
        .open(&path)?;

    file.seek(SeekFrom::Start(offset))?;
    file.write_all(chunk)?;

    let received = offset + chunk.len() as u64;
    let percent = if total_size == 0 {
        100
    } else {
        ((received as f64 / total_size as f64) * 100.0).round() as u8
    };

    let _ = app.emit(
        "file-receive-progress",
        format!(
            "{}|{}|{}|{}|{}",
            transfer_id, safe_filename, received, total_size, percent
        ),
    );

    if received >= total_size {
        if let Some(session) = sessions().lock().unwrap().get_mut(&transfer_id) {
            session.file_path = path_string.clone();
        }

        let _ = app.emit("file-received", format!("{}|{}", transfer_id, path_string));
    }

    Ok(path)
}
