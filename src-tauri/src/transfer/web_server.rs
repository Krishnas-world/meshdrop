use std::fs::File;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use tauri::{AppHandle, Emitter};

static WEB_SERVER_RUNNING: AtomicBool = AtomicBool::new(false);
static ACTIVE_SHARE_PATH: OnceLock<Mutex<Option<PathBuf>>> = OnceLock::new();
static WEB_SHARE_ENABLED: AtomicBool = AtomicBool::new(false);

use std::sync::OnceLock;

fn share_path() -> &'static Mutex<Option<PathBuf>> {
    ACTIVE_SHARE_PATH.get_or_init(|| Mutex::new(None))
}

pub fn start_web_server(app: AppHandle, file_path: String) -> Result<String, String> {
    if !file_path.is_empty() {
        let path = PathBuf::from(&file_path);
        if !path.exists() || !path.is_file() {
            return Err("Target file does not exist or is invalid".to_string());
        }
        // Save the active path
        *share_path().lock().unwrap() = Some(path.clone());
        WEB_SHARE_ENABLED.store(true, Ordering::SeqCst);
    }

    if WEB_SERVER_RUNNING.swap(true, Ordering::SeqCst) {
        // Server already running, just updated active path
        return get_local_ip_url();
    }

    let listen_app = app.clone();
    thread::spawn(move || {
        let listener = match TcpListener::bind("0.0.0.0:7880") {
            Ok(l) => l,
            Err(e) => {
                println!("Failed to bind web server to 7880: {}", e);
                WEB_SERVER_RUNNING.store(false, Ordering::SeqCst);
                return;
            }
        };

        println!("Web Share server active on http://0.0.0.0:7880");

        for stream in listener.incoming() {
            if !WEB_SERVER_RUNNING.load(Ordering::SeqCst) {
                break;
            }

            match stream {
                Ok(mut stream) => {
                    let _ = stream.set_nodelay(true);
                    let app_clone = listen_app.clone();
                    thread::spawn(move || {
                        if let Err(e) = handle_connection(app_clone, &mut stream) {
                            println!("Error handling web connection: {}", e);
                        }
                    });
                }
                Err(e) => {
                    println!("Web connection failed: {}", e);
                }
            }
        }
    });

    get_local_ip_url()
}

pub fn stop_web_server() {
    // Keep HTTP server running, just clear the shared file path and disable share route
    *share_path().lock().unwrap() = None;
    WEB_SHARE_ENABLED.store(false, Ordering::SeqCst);
}

fn get_local_ip_url() -> Result<String, String> {
    use std::net::ToSocketAddrs;

    let mut ips = Vec::new();

    // 1. Try resolving via UDP connection to broadcast (resolves interface used for default routing)
    if let Ok(socket) = std::net::UdpSocket::bind("0.0.0.0:0") {
        if socket.connect("255.255.255.255:80").is_ok() {
            if let Ok(addr) = socket.local_addr() {
                let ip_str = addr.ip().to_string();
                if ip_str != "127.0.0.1" && ip_str != "0.0.0.0" {
                    ips.push(ip_str);
                }
            }
        }
    }

    // 2. Query all IPs registered to this computer's hostname
    let hostname = std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "localhost".to_string());
    
    if let Ok(addrs) = format!("{}:0", hostname).to_socket_addrs() {
        for addr in addrs {
            if addr.ip().is_ipv4() && !addr.ip().is_loopback() {
                let ip_str = addr.ip().to_string();
                if !ips.contains(&ip_str) {
                    ips.push(ip_str);
                }
            }
        }
    }

    let resolved_ip = if ips.is_empty() {
        "127.0.0.1".to_string()
    } else {
        // Sort IPs: prioritize physical networks (192.168.x.x, 10.x.x.x) over virtual adapters (WSL, Docker, Cloudflare WARP)
        ips.sort_by(|a, b| {
            let score = |ip: &str| {
                if ip.starts_with("192.168.") {
                    1
                } else if ip.starts_with("10.") {
                    2
                } else if ip.starts_with("172.") && !ip.starts_with("172.16.") {
                    3
                } else {
                    4 // Virtual interfaces like Cloudflare WARP (172.16.0.x) or WSL
                }
            };
            score(a).cmp(&score(b))
        });
        ips[0].clone()
    };

    let sender_slug = clean_device_name(&hostname);
    Ok(format!("http://{}:7880/share/{}-browser", resolved_ip, sender_slug))
}

fn clean_device_name(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() { c.to_ascii_lowercase() } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<&str>>()
        .join("-")
}

fn parse_user_agent(ua: &str) -> String {
    let is_android = ua.contains("Android");
    let is_iphone = ua.contains("iPhone") || ua.contains("iPad");
    let is_windows = ua.contains("Windows");
    let is_mac = ua.contains("Macintosh") && !is_iphone;
    let is_linux = ua.contains("Linux") && !is_android;

    let os = if is_android {
        "Android Phone"
    } else if is_iphone {
        "iPhone"
    } else if is_windows {
        "Windows PC"
    } else if is_mac {
        "Mac"
    } else if is_linux {
        "Linux PC"
    } else {
        "Unknown Device"
    };

    let browser = if ua.contains("Firefox") {
        "Firefox"
    } else if ua.contains("Chrome") {
        "Chrome"
    } else if ua.contains("Safari") && !ua.contains("Chrome") {
        "Safari"
    } else if ua.contains("Edge") {
        "Edge"
    } else {
        "Browser"
    };

    format!("{} ({})", os, browser)
}

fn handle_connection(app: AppHandle, stream: &mut TcpStream) -> std::io::Result<()> {
    let mut buffer = [0; 1024];
    let bytes_read = stream.read(&mut buffer)?;
    let request = String::from_utf8_lossy(&buffer[..bytes_read]);

    let first_line = request.lines().next().unwrap_or("");
    let parts: Vec<&str> = first_line.split_whitespace().collect();
    if parts.len() < 2 {
        return Ok(());
    }

    let method = parts[0];
    let uri = parts[1];

    // Find User-Agent header
    let mut user_agent = "Unknown User-Agent".to_string();
    for line in request.lines() {
        if line.to_lowercase().starts_with("user-agent:") {
            user_agent = line["user-agent:".len()..].trim().to_string();
            break;
        }
    }

    let client_ip = stream.peer_addr().map(|addr| addr.ip().to_string()).unwrap_or_else(|_| "Unknown IP".to_string());
    let device_details = parse_user_agent(&user_agent);

    if method == "GET" {
        if uri == "/" {
            serve_product_landing_page(stream)?;
        } else if uri.starts_with("/share/") {
            // Check if file sharing is active and enabled, if not redirect to root
            let has_active_file = share_path().lock().unwrap().is_some() && WEB_SHARE_ENABLED.load(Ordering::SeqCst);
            if has_active_file {
                serve_share_page(stream)?;
            } else {
                send_redirect(stream, "/")?;
            }
        } else if uri == "/download" {
            serve_download(app, stream, client_ip, device_details)?;
        } else if uri.starts_with("/download-app") {
            serve_app_download(app, stream, uri)?;
        } else {
            send_404(stream)?;
        }
    }

    Ok(())
}

fn send_redirect(stream: &mut TcpStream, location: &str) -> std::io::Result<()> {
    let response = format!(
        "HTTP/1.1 302 Found\r\n\
         Location: {}\r\n\
         Content-Length: 0\r\n\
         Connection: close\r\n\r\n",
        location
    );
    stream.write_all(response.as_bytes())
}

fn serve_product_landing_page(stream: &mut TcpStream) -> std::io::Result<()> {
    let body = r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>MeshDrop - Peer-to-Peer Local File Sharing</title>
    <style>
        body {
            font-family: 'Segoe UI', system-ui, sans-serif;
            background: linear-gradient(135deg, #f7f9f5 0%, #edf4f0 52%, #eef1f7 100%);
            margin: 0;
            min-height: 100vh;
            display: grid;
            place-items: center;
            color: #17201b;
        }
        .card {
            background: rgba(255, 255, 255, 0.9);
            padding: 36px;
            border-radius: 16px;
            box-shadow: 0 20px 50px rgba(42, 55, 48, 0.12);
            border: 1px solid rgba(23, 32, 27, 0.08);
            max-width: 500px;
            width: 100%;
            text-align: center;
            backdrop-filter: blur(12px);
            box-sizing: border-box;
        }
        .eyebrow {
            color: #1f7a59;
            font-size: 0.8rem;
            font-weight: 800;
            text-transform: uppercase;
            letter-spacing: 0.05em;
            margin-bottom: 8px;
        }
        h1 {
            font-size: 2.5rem;
            margin: 0 0 12px;
            line-height: 1.1;
        }
        .tagline {
            color: #536158;
            font-size: 1rem;
            margin-bottom: 28px;
            line-height: 1.4;
        }
        .features {
            display: grid;
            gap: 16px;
            text-align: left;
            margin-bottom: 32px;
        }
        .feature-item {
            display: flex;
            align-items: flex-start;
            gap: 12px;
        }
        .feature-icon {
            font-size: 1.25rem;
            flex-shrink: 0;
        }
        .feature-text strong {
            display: block;
            font-size: 0.95rem;
            color: #17201b;
        }
        .feature-text span {
            font-size: 0.85rem;
            color: #607068;
        }
        .btn {
            display: inline-block;
            width: 100%;
            background: #17201b;
            color: #ffffff;
            text-decoration: none;
            padding: 14px 20px;
            border-radius: 10px;
            font-weight: 900;
            box-shadow: 0 8px 20px rgba(23, 32, 27, 0.2);
            transition: all 0.2s ease;
            box-sizing: border-box;
            cursor: pointer;
            border: none;
            font-size: 1rem;
        }
        .btn:hover {
            transform: translateY(-2px);
            box-shadow: 0 12px 28px rgba(23, 32, 27, 0.3);
            background: #1f7a59;
        }
        .get-app-section {
            background: #fbfbfc;
            border: 1px solid rgba(53, 91, 169, 0.15);
            border-radius: 12px;
            padding: 18px;
            text-align: left;
            box-sizing: border-box;
            width: 100%;
        }
        .get-app-section h3 {
            margin: 0 0 6px;
            font-size: 1.1rem;
            color: #17201b;
        }
        .get-app-section p {
            margin: 0 0 14px;
            font-size: 0.88rem;
            color: #607068;
            font-weight: 600;
        }
        .secondary-btn {
            background: #355ba9 !important;
        }
        .secondary-btn:hover {
            background: #254482 !important;
        }
    </style>
</head>
<body>
    <div class="card">
        <div class="eyebrow">Local Peer-to-Peer Sharing</div>
        <h1>MeshDrop</h1>
        <p class="tagline">Share files directly between your devices at maximum WiFi speeds. Zero cloud storage required, fully private.</p>
        
        <div class="features">
            <div class="feature-item">
                <span class="feature-icon">⚡</span>
                <div class="feature-text">
                    <strong>Maximum Speeds</strong>
                    <span>Transfers files directly over WiFi Direct or local LAN interfaces.</span>
                </div>
            </div>
            <div class="feature-item">
                <span class="feature-icon">🔒</span>
                <div class="feature-text">
                    <strong>100% Private & Offline</strong>
                    <span>No cloud servers. Transfers happen directly between devices locally.</span>
                </div>
            </div>
            <div class="feature-item">
                <span class="feature-icon">🌐</span>
                <div class="feature-text">
                    <strong>Multi-OS Ecosystem</strong>
                    <span>Native apps for desktop with cross-device browser support.</span>
                </div>
            </div>
        </div>

        <div class="get-app-section">
            <div class="eyebrow" style="color: #355ba9; margin-bottom: 4px;">MeshDrop App</div>
            <h3>Install Native Client</h3>
            <p id="osDetect">Detecting your system...</p>
            <a id="appDownloadBtn" href="/download-app" class="btn secondary-btn" style="display: none;">Download Installer</a>
        </div>
    </div>

    <script>
        window.onload = function() {
            const ua = navigator.userAgent;
            let os = "Unknown OS";
            let osParam = "unknown";
            let ext = "bin";

            if (ua.indexOf("Windows") !== -1) {
                os = "Windows";
                osParam = "windows";
                ext = "msi";
            } else if (ua.indexOf("Android") !== -1) {
                os = "Android";
                osParam = "android";
                ext = "apk";
            } else if (ua.indexOf("Mac") !== -1 || ua.indexOf("iPhone") !== -1 || ua.indexOf("iPad") !== -1) {
                os = "macOS / iOS";
                osParam = "macos";
                ext = "dmg";
            } else if (ua.indexOf("Linux") !== -1) {
                os = "Linux";
                osParam = "linux";
                ext = "deb";
            }

            const osDetect = document.getElementById("osDetect");
            const appDownloadBtn = document.getElementById("appDownloadBtn");

            osDetect.textContent = "Detected System: " + os;
            appDownloadBtn.href = "/download-app?os=" + osParam;
            appDownloadBtn.textContent = "Download MeshDrop for " + os + " (." + ext + ")";
            appDownloadBtn.style.display = "inline-block";
        };
    </script>
</body>
</html>"#;

    let response = format!(
        "HTTP/1.1 200 OK\r\n\
         Content-Type: text/html\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\r\n\
         {}",
        body.len(),
        body
    );

    stream.write_all(response.as_bytes())?;
    Ok(())
}

fn serve_share_page(stream: &mut TcpStream) -> std::io::Result<()> {
    let opt_path = share_path().lock().unwrap().clone();
    let (filename, size_str) = match opt_path {
        Some(p) => {
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("meshdrop-file").to_string();
            let size = std::fs::metadata(&p).map(|m| m.len()).unwrap_or(0);
            let mb = size as f64 / 1_048_576.0;
            (name, format!("{:.2} MB", mb))
        }
        None => ("No active share".to_string(), "0 MB".to_string()),
    };

    let body = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>MeshDrop - Web Share</title>
    <style>
        body {{
            font-family: 'Segoe UI', system-ui, sans-serif;
            background: linear-gradient(135deg, #f7f9f5 0%, #edf4f0 52%, #eef1f7 100%);
            margin: 0;
            min-height: 100vh;
            display: grid;
            place-items: center;
            color: #17201b;
        }}
        .card {{
            background: rgba(255, 255, 255, 0.9);
            padding: 32px;
            border-radius: 16px;
            box-shadow: 0 20px 50px rgba(42, 55, 48, 0.12);
            border: 1px solid rgba(23, 32, 27, 0.08);
            max-width: 440px;
            width: 100%;
            text-align: center;
            backdrop-filter: blur(12px);
            box-sizing: border-box;
        }}
        .eyebrow {{
            color: #1f7a59;
            font-size: 0.8rem;
            font-weight: 800;
            text-transform: uppercase;
            letter-spacing: 0.05em;
            margin-bottom: 8px;
        }}
        h1 {{
            font-size: 2.2rem;
            margin: 0 0 16px;
            line-height: 1.1;
        }}
        .file-info {{
            background: #f1f6f2;
            border: 1px solid rgba(31, 122, 89, 0.15);
            border-radius: 12px;
            padding: 18px;
            margin: 24px 0;
            text-align: left;
        }}
        .file-name {{
            font-weight: 700;
            font-size: 1.1rem;
            word-break: break-all;
            margin-bottom: 4px;
            color: #17201b;
        }}
        .file-size {{
            color: #607068;
            font-size: 0.9rem;
            font-weight: 600;
        }}
        .btn {{
            display: inline-block;
            width: 100%;
            background: #17201b;
            color: #ffffff;
            text-decoration: none;
            padding: 14px 20px;
            border-radius: 10px;
            font-weight: 900;
            box-shadow: 0 8px 20px rgba(23, 32, 27, 0.2);
            transition: all 0.2s ease;
            box-sizing: border-box;
            cursor: pointer;
            border: none;
            font-size: 1rem;
        }}
        .btn:hover {{
            transform: translateY(-2px);
            box-shadow: 0 12px 28px rgba(23, 32, 27, 0.3);
            background: #1f7a59;
        }}
        .progress-box {{
            display: none;
            margin-top: 10px;
            text-align: left;
        }}
        .progress-track {{
            height: 12px;
            background: #e2e8e3;
            border-radius: 6px;
            overflow: hidden;
            position: relative;
            margin-bottom: 8px;
        }}
        .progress-bar {{
            height: 100%;
            background: linear-gradient(90deg, #1f7a59, #355ba9);
            width: 0%;
            transition: width 0.1s linear;
        }}
        .progress-stats {{
            display: flex;
            justify-content: space-between;
            font-size: 0.85rem;
            color: #607068;
            font-weight: bold;
        }}
        .divider {{
            height: 1px;
            background: rgba(23, 32, 27, 0.08);
            margin: 28px 0;
        }}
        .get-app-section {{
            background: #fbfbfc;
            border: 1px solid rgba(53, 91, 169, 0.15);
            border-radius: 12px;
            padding: 18px;
            text-align: left;
            box-sizing: border-box;
        }}
        .get-app-section h3 {{
            margin: 0 0 6px;
            font-size: 1.1rem;
            color: #17201b;
        }}
        .get-app-section p {{
            margin: 0 0 14px;
            font-size: 0.88rem;
            color: #607068;
            font-weight: 600;
        }}
        .secondary-btn {{
            background: #355ba9 !important;
        }}
        .secondary-btn:hover {{
            background: #254482 !important;
        }}
    </style>
</head>
<body>
    <div class="card">
        <div class="eyebrow">MeshDrop Transfer</div>
        <h1>Incoming File</h1>
        <div class="file-info">
            <div class="file-name">{}</div>
            <div class="file-size">{}</div>
        </div>
        <a href="/download" class="btn" style="margin-bottom: 12px; display: block; text-align: center; line-height: 1.2;">Download Natively (Fastest)</a>
        <button id="downloadBtn" onclick="startDownload()" class="btn" style="background: transparent; color: #17201b; border: 1px solid rgba(23, 32, 27, 0.2); box-shadow: none;">Download with Page Progress Bar</button>
        
        <div id="progressBox" class="progress-box">
            <div class="progress-track">
                <div id="progressBar" class="progress-bar"></div>
            </div>
            <div class="progress-stats">
                <span id="progressPercent">0%</span>
                <span id="speedText">0.00 MB/s</span>
            </div>
        </div>

        <div class="divider"></div>
        
        <div class="get-app-section">
            <div class="eyebrow" style="color: #355ba9;">MeshDrop Ecosystem</div>
            <h3>Install Native App</h3>
            <p id="osDetect">Detecting your system...</p>
            <a id="appDownloadBtn" href="/download-app" class="btn secondary-btn" style="display: none;">Download Installer</a>
        </div>
    </div>

    <script>
        window.onload = function() {{
            const ua = navigator.userAgent;
            let os = "Unknown OS";
            let osParam = "unknown";
            let ext = "bin";

            if (ua.indexOf("Windows") !== -1) {{
                os = "Windows";
                osParam = "windows";
                ext = "msi";
            }} else if (ua.indexOf("Android") !== -1) {{
                os = "Android";
                osParam = "android";
                ext = "apk";
            }} else if (ua.indexOf("Mac") !== -1 || ua.indexOf("iPhone") !== -1 || ua.indexOf("iPad") !== -1) {{
                os = "macOS / iOS";
                osParam = "macos";
                ext = "dmg";
            }} else if (ua.indexOf("Linux") !== -1) {{
                os = "Linux";
                osParam = "linux";
                ext = "deb";
            }}

            const osDetect = document.getElementById("osDetect");
            const appDownloadBtn = document.getElementById("appDownloadBtn");

            osDetect.textContent = "Detected System: " + os;
            appDownloadBtn.href = "/download-app?os=" + osParam;
            appDownloadBtn.textContent = "Download MeshDrop for " + os + " (." + ext + ")";
            appDownloadBtn.style.display = "inline-block";
        }};

        async function startDownload() {{
            const btn = document.getElementById('downloadBtn');
            const progressBox = document.getElementById('progressBox');
            const progressBar = document.getElementById('progressBar');
            const progressPercent = document.getElementById('progressPercent');
            const speedText = document.getElementById('speedText');

            btn.style.display = 'none';
            progressBox.style.display = 'block';

            try {{
                const response = await fetch('/download');
                if (!response.ok) throw new Error('Download failed');

                const reader = response.body.getReader();
                const contentLength = +response.headers.get('Content-Length') || 0;

                let receivedLength = 0;
                let chunks = [];
                let startTime = Date.now();

                while(true) {{
                    const {{done, value}} = await reader.read();
                    if (done) break;

                    chunks.push(value);
                    receivedLength += value.length;

                    if (contentLength > 0) {{
                        const percent = Math.round((receivedLength / contentLength) * 100);
                        progressBar.style.width = percent + '%';
                        progressPercent.textContent = percent + '%';
                    }}

                    const elapsedSecs = (Date.now() - startTime) / 1000;
                    if (elapsedSecs > 0) {{
                        const speedMB = (receivedLength / 1048576) / elapsedSecs;
                        speedText.textContent = speedMB.toFixed(2) + ' MB/s';
                    }}
                }}

                // Save triggers
                const blob = new Blob(chunks);
                const url = URL.createObjectURL(blob);
                const a = document.createElement('a');
                a.href = url;
                a.download = "{}";
                document.body.appendChild(a);
                a.click();
                a.remove();
                URL.revokeObjectURL(url);

                progressPercent.textContent = 'Complete!';
                speedText.textContent = 'Saved successfully';
            }} catch(err) {{
                alert(err.message);
                btn.style.display = 'block';
                progressBox.style.display = 'none';
            }}
        }}
    </script>
</body>
</html>"#,
        filename, size_str, filename
    );

    let response = format!(
        "HTTP/1.1 200 OK\r\n\
         Content-Type: text/html\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\r\n\
         {}",
        body.len(),
        body
    );

    stream.write_all(response.as_bytes())?;
    Ok(())
}

fn serve_download(app: AppHandle, stream: &mut TcpStream, client_ip: String, device_details: String) -> std::io::Result<()> {
    if !WEB_SHARE_ENABLED.load(Ordering::SeqCst) {
        send_404(stream)?;
        return Ok(());
    }

    let opt_path = share_path().lock().unwrap().clone();
    let path = match opt_path {
        Some(p) => p,
        None => {
            send_404(stream)?;
            return Ok(());
        }
    };

    let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("meshdrop-file");
    let mut file = match File::open(&path) {
        Ok(f) => f,
        Err(e) => {
            println!("Failed to open file: {}", e);
            send_500(stream)?;
            return Ok(());
        }
    };

    let size = match file.metadata() {
        Ok(m) => m.len(),
        Err(_) => 0,
    };

    // Emit event with client details: filename|client_ip|device_details
    let event_payload = format!("{}|{}|{}", filename, client_ip, device_details);
    let _ = app.emit("web-share-download-start", event_payload);

    let headers = format!(
        "HTTP/1.1 200 OK\r\n\
         Content-Type: application/octet-stream\r\n\
         Content-Length: {}\r\n\
         Content-Disposition: attachment; filename=\"{}\"\r\n\
         Connection: close\r\n\r\n",
         size, filename
    );

    stream.write_all(headers.as_bytes())?;

    let mut buffer = [0; 64 * 1024];
    let mut bytes_written = 0u64;
    let mut last_percent = 0u32;
    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        stream.write_all(&buffer[..bytes_read])?;
        bytes_written += bytes_read as u64;

        if size > 0 {
            let percent = (bytes_written * 100 / size) as u32;
            if percent != last_percent {
                last_percent = percent;
                let _ = app.emit("web-share-download-progress", percent);
            }
        }
    }

    let _ = app.emit("web-share-download-complete", filename.to_string());

    Ok(())
}

fn serve_app_download(app: AppHandle, stream: &mut TcpStream, uri: &str) -> std::io::Result<()> {
    let os = if uri.contains("os=windows") {
        "windows"
    } else if uri.contains("os=macos") {
        "macos"
    } else if uri.contains("os=linux") {
        "linux"
    } else if uri.contains("os=android") {
        "android"
    } else {
        "unknown"
    };

    let (filename, _ext) = match os {
        "windows" => ("meshdrop.msi", "msi"),
        "macos" => ("meshdrop.dmg", "dmg"),
        "linux" => ("meshdrop.deb", "deb"),
        "android" => ("meshdrop.apk", "apk"),
        _ => ("meshdrop.bin", "bin"),
    };

    let _ = app.emit("web-share-app-download-start", format!("{}|{}", os, filename));

    // Try to load from "installers/" folder in current directory
    let mut installer_path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    installer_path.push("installers");
    installer_path.push(filename);

    if installer_path.exists() && installer_path.is_file() {
        let mut file = match File::open(&installer_path) {
            Ok(f) => f,
            Err(_) => {
                send_500(stream)?;
                return Ok(());
            }
        };
        let size = file.metadata().map(|m| m.len()).unwrap_or(0);
        let headers = format!(
            "HTTP/1.1 200 OK\r\n\
             Content-Type: application/octet-stream\r\n\
             Content-Length: {}\r\n\
             Content-Disposition: attachment; filename=\"{}\"\r\n\
             Connection: close\r\n\r\n",
            size, filename
        );
        stream.write_all(headers.as_bytes())?;
        let mut buffer = [0; 64 * 1024];
        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            stream.write_all(&buffer[..bytes_read])?;
        }
    } else {
        // Fallback: serve a 500KB dummy executable/installer binary (helps locally test the download button)
        let dummy_size = 500_000;
        let headers = format!(
            "HTTP/1.1 200 OK\r\n\
             Content-Type: application/octet-stream\r\n\
             Content-Length: {}\r\n\
             Content-Disposition: attachment; filename=\"{}\"\r\n\
             Connection: close\r\n\r\n",
            dummy_size, filename
        );
        stream.write_all(headers.as_bytes())?;
        
        // Write mock binary header & padding
        let header_text = format!("MeshDrop Native {} App Build Installer Placeholder. Place your compiled files in installers/{} to serve a real binary.", os, filename);
        let mut bytes_written = 0;
        let header_bytes = header_text.as_bytes();
        stream.write_all(header_bytes)?;
        bytes_written += header_bytes.len();
        
        let chunk = [0u8; 1024];
        while bytes_written < dummy_size {
            let to_write = std::cmp::min(chunk.len(), dummy_size - bytes_written);
            stream.write_all(&chunk[..to_write])?;
            bytes_written += to_write;
        }
    }

    Ok(())
}

fn send_404(stream: &mut TcpStream) -> std::io::Result<()> {
    let response = "HTTP/1.1 404 NOT FOUND\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
    stream.write_all(response.as_bytes())
}

fn send_500(stream: &mut TcpStream) -> std::io::Result<()> {
    let response = "HTTP/1.1 500 INTERNAL SERVER ERROR\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
    stream.write_all(response.as_bytes())
}
