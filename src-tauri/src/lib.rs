mod transfer;
use std::path::Path;
use transfer::discovery;
use transfer::file_response_sender;
use transfer::file_sender;
use transfer::receiver;
use transfer::sender;
use transfer::transport;
#[tauri::command]
fn start_server(app: tauri::AppHandle) {
    receiver::start_server(app);
}

#[tauri::command]
fn start_discovery(app: tauri::AppHandle) -> Result<String, String> {
    discovery::start_discovery(app).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_transport_plan() -> Vec<transport::TransportOption> {
    transport::transport_plan()
}

#[tauri::command]
fn start_direct_connect(app: tauri::AppHandle) -> Result<Vec<transport::TransportOption>, String> {
    transport::start_direct_connect(app).map_err(|e| e.to_string())
}

#[tauri::command]
fn send_message(ip: String, message: String) {
    sender::send_message(ip, message);
}
#[tauri::command]
fn send_file_offer(ip: String, filename: String, filesize: u64) -> Result<String, String> {
    file_sender::send_file_offer(ip, filename, filesize).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_file_info(path: String) -> Result<String, String> {
    let path = Path::new(&path);
    let filename = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| "Invalid file path".to_string())?;
    let size = std::fs::metadata(path).map_err(|e| e.to_string())?.len();

    Ok(format!("{}|{}", filename, size))
}

#[tauri::command]
fn send_file_data(
    ip: String,
    transfer_id: String,
    filename: String,
    bytes: Vec<u8>,
) -> Result<(), String> {
    file_sender::send_file_data(ip, transfer_id, filename, bytes).map_err(|e| e.to_string())
}

#[tauri::command]
fn send_file_from_path(
    app: tauri::AppHandle,
    ip: String,
    transfer_id: String,
    path: String,
) -> Result<(), String> {
    file_sender::send_file_from_path(app, ip, transfer_id, path).map_err(|e| e.to_string())
}

#[tauri::command]
fn send_file_accept(ip: String, transfer_id: String) {
    file_response_sender::send_file_accept(ip, transfer_id);
}

#[tauri::command]
fn send_file_reject(ip: String, transfer_id: String) {
    file_response_sender::send_file_reject(ip, transfer_id);
}

#[tauri::command]
fn open_system_settings(action: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let url = match action.as_str() {
            "bluetooth-settings" => "ms-settings:bluetooth",
            "wifi-settings" => "ms-settings:network-wifi",
            "hotspot-settings" => "ms-settings:network-mobilehotspot",
            _ => return Err(format!("Unsupported action: {}", action)),
        };
        std::process::Command::new("cmd")
            .args(&["/c", "start", url])
            .spawn()
            .map_err(|e| e.to_string())?;
        Ok(())
    }
    #[cfg(target_os = "macos")]
    {
        let path = match action.as_str() {
            "bluetooth-settings" => "/System/Library/PreferencePanes/Bluetooth.prefPane",
            "wifi-settings" => "/System/Library/PreferencePanes/Network.prefPane",
            "hotspot-settings" => "/System/Library/PreferencePanes/SharingPref.prefPane",
            _ => return Err(format!("Unsupported action: {}", action)),
        };
        std::process::Command::new("open")
            .arg(path)
            .spawn()
            .map_err(|e| e.to_string())?;
        Ok(())
    }
    #[cfg(target_os = "linux")]
    {
        let args = match action.as_str() {
            "bluetooth-settings" => vec!["bluetooth"],
            "wifi-settings" => vec!["wifi"],
            "hotspot-settings" => vec!["network"],
            _ => return Err(format!("Unsupported action: {}", action)),
        };
        std::process::Command::new("gnome-control-center")
            .args(&args)
            .spawn()
            .or_else(|_| {
                std::process::Command::new("nm-connection-editor")
                    .spawn()
            })
            .map_err(|e| format!("Could not open system settings: {}", e))?;
        Ok(())
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        Err("Unsupported OS for settings shortcuts".to_string())
    }
}

#[tauri::command]
fn get_receive_folder() -> String {
    transfer::file_receiver::get_receive_folder()
}

#[tauri::command]
fn set_receive_folder(path: String) -> Result<String, String> {
    transfer::file_receiver::set_receive_folder(path).map_err(|e| e.to_string())
}

#[tauri::command]
fn set_receive_location(path: String) -> Result<String, String> {
    transfer::file_receiver::set_receive_location(path).map_err(|e| e.to_string())
}

#[tauri::command]
fn start_web_share(app: tauri::AppHandle, path: String) -> Result<String, String> {
    transfer::web_server::start_web_server(app, path)
}

#[tauri::command]
fn stop_web_share() {
    transfer::web_server::stop_web_server();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let handle = app.handle().clone();
            let _ = transfer::web_server::start_web_server(handle, String::new());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            start_server,
            start_discovery,
            get_transport_plan,
            start_direct_connect,
            send_message,
            send_file_offer,
            get_file_info,
            send_file_data,
            send_file_from_path,
            send_file_accept,
            send_file_reject,
            get_receive_folder,
            set_receive_folder,
            set_receive_location,
            open_system_settings,
            start_web_share,
            stop_web_share
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
