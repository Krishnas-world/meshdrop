use std::net::UdpSocket;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use uuid::Uuid;

const DISCOVERY_PORT: u16 = 7879;
const TRANSFER_PORT: u16 = 7878;
const DISCOVERY_MAGIC: &str = "MESHDROP_DISCOVERY_V1";

static DISCOVERY_STARTED: AtomicBool = AtomicBool::new(false);
static DEVICE_ID: OnceLock<String> = OnceLock::new();

fn device_id() -> &'static str {
    DEVICE_ID.get_or_init(|| Uuid::new_v4().to_string())
}

fn device_name() -> String {
    std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "MeshDrop Device".to_string())
}

pub fn start_discovery(app: AppHandle) -> std::io::Result<String> {
    if DISCOVERY_STARTED.swap(true, Ordering::SeqCst) {
        return Ok(device_name());
    }

    let name = device_name();
    let announce_name = name.clone();
    let listen_app = app.clone();

    thread::spawn(move || {
        let socket = match UdpSocket::bind("0.0.0.0:0") {
            Ok(socket) => socket,
            Err(e) => {
                println!("Discovery announce bind failed: {}", e);
                return;
            }
        };

        if let Err(e) = socket.set_broadcast(true) {
            println!("Discovery broadcast enable failed: {}", e);
            return;
        }

        loop {
            let payload = format!(
                "{}|{}|{}|{}",
                DISCOVERY_MAGIC,
                device_id(),
                announce_name,
                TRANSFER_PORT
            );

            if let Err(e) = socket.send_to(
                payload.as_bytes(),
                format!("255.255.255.255:{}", DISCOVERY_PORT),
            ) {
                println!("Discovery announce failed: {}", e);
            }

            thread::sleep(Duration::from_secs(2));
        }
    });

    thread::spawn(move || {
        let socket = match UdpSocket::bind(format!("0.0.0.0:{}", DISCOVERY_PORT)) {
            Ok(socket) => socket,
            Err(e) => {
                println!("Discovery listen bind failed: {}", e);
                return;
            }
        };

        let mut buffer = [0_u8; 1024];

        loop {
            let (size, peer) = match socket.recv_from(&mut buffer) {
                Ok(result) => result,
                Err(e) => {
                    println!("Discovery receive failed: {}", e);
                    continue;
                }
            };

            let message = String::from_utf8_lossy(&buffer[..size]);
            let parts: Vec<&str> = message.split('|').collect();

            if parts.len() != 4 || parts[0] != DISCOVERY_MAGIC || parts[1] == device_id() {
                continue;
            }

            let payload = format!("{}|{}|{}|{}", parts[1], parts[2], peer.ip(), parts[3]);
            let _ = listen_app.emit("device-discovered", payload);
        }
    });

    Ok(name)
}
