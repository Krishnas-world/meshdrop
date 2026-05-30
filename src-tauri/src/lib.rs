use std::io::Read;
use std::net::TcpListener;
use std::thread;
use std::net::TcpStream;
use std::io::Write;

#[tauri::command]
fn start_server() {
    thread::spawn(|| {
        let listener = match TcpListener::bind("0.0.0.0:7878") {
            Ok(listener) => listener,
            Err(e) => {
                println!("Could not start server: {}", e);
                return;
            }
        };

        println!("Server running on port 7878");

        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    println!("New connection!");

                    let mut buffer = [0; 1024];

                    match stream.read(&mut buffer) {
                        Ok(bytes_read) => {
                            let message = String::from_utf8_lossy(&buffer[..bytes_read]);

                            println!("Received: {}", message);
                        }

                        Err(e) => {
                            println!("Read error: {}", e);
                        }
                    }
                }

                Err(e) => {
                    println!("Connection failed: {}", e);
                }
            }
        }
    });
}

#[tauri::command]
fn send_message(ip: String, message: String) {

    match TcpStream::connect(format!("{}:7878", ip)) {

        Ok(mut stream) => {

            stream.write_all(message.as_bytes())
                .expect("Failed to send");

            println!("Message sent");
        }

        Err(e) => {
            println!("Connection failed: {}", e);
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![start_server, send_message])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
