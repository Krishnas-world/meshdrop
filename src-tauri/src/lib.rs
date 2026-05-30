mod transfer;
use transfer::file_response_sender;
use transfer::file_sender;
use transfer::receiver;
use transfer::sender;
#[tauri::command]
fn start_server(app: tauri::AppHandle) {
    receiver::start_server(app);
}

#[tauri::command]
fn send_message(ip: String, message: String) {
    sender::send_message(ip, message);
}
#[tauri::command]
fn send_file_offer(ip: String, filename: String, filesize: u64) {
    file_sender::send_file_offer(ip, filename, filesize);
}

#[tauri::command]
fn send_file_accept(ip: String) {
    file_response_sender::send_file_accept(ip);
}

#[tauri::command]
fn send_file_reject(ip: String) {
    file_response_sender::send_file_reject(ip);
}
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            start_server,
            send_message,
            send_file_offer,
            send_file_accept,
            send_file_reject
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
