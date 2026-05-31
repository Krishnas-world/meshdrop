use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use tauri::{AppHandle, Emitter};

static RECEIVE_FOLDER: OnceLock<Mutex<PathBuf>> = OnceLock::new();

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

pub fn handle_file_offer(app: &AppHandle, filename: String, filesize: u64) {
    let payload = format!("{}|{}", filename, filesize);

    let _ = app.emit("incoming-file-offer", payload);
}

pub fn handle_file_data(app: &AppHandle, payload: &[u8]) -> std::io::Result<PathBuf> {
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
