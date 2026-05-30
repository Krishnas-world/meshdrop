use super::protocol::{read_packet, PacketType};
use std::net::TcpListener;
use std::thread;
use tauri::Emitter;

pub fn start_server(app: tauri::AppHandle) {
    thread::spawn(move || {
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

                    match read_packet(&mut stream) {
                        Ok(packet) => match packet.packet_type {
                            PacketType::Message => {
                                let message = String::from_utf8_lossy(&packet.payload);

                                println!("Received Message: {}", message);

                                let _ = app.emit("message-received", message.to_string());
                            }

                            PacketType::FileOffer => {
                                let payload = String::from_utf8_lossy(&packet.payload);

                                let parts: Vec<&str> = payload.split('|').collect();

                                if parts.len() == 3 {
                                    let transfer_id = parts[0].to_string();

                                    let filename = parts[1].to_string();

                                    let filesize = parts[2].parse::<u64>().unwrap_or(0);

                                    println!(
                                        "Offer received: {} {} {}",
                                        transfer_id, filename, filesize
                                    );

                                    crate::transfer::file_receiver::handle_file_offer(
                                        &app, filename, filesize,
                                    );
                                }
                            }

                            PacketType::FileData => {
                                println!("Received FileData packet");
                            }

                            PacketType::FileAccept => {
                                println!("File Accepted");

                                let _ = app.emit("file-accepted", "accepted");
                            }

                            PacketType::FileReject => {
                                println!("File Rejected");

                                let _ = app.emit("file-rejected", "rejected");
                            }
                        },

                        Err(e) => {
                            println!("Packet read error: {}", e);
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
