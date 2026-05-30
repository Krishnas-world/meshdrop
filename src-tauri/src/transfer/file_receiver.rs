use tauri::{AppHandle, Emitter};

pub fn handle_file_offer(
    app: &AppHandle,
    filename: String,
    filesize: u64,
) {
    let payload =
        format!("{}|{}", filename, filesize);

    let _ =
        app.emit("incoming-file-offer", payload);
}