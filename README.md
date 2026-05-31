# 🌐 MeshDrop

MeshDrop is a premium, secure, and 100% offline peer-to-peer (P2P) file sharing system designed to transfer files between your local devices at maximum network hardware speeds. Zero cloud storage required, fully private, and cross-platform.

---

## ✨ Core Features

* ⚡ **Maximum LAN/WiFi Speeds**: Stream files directly over your local home network, WiFi Direct, or local hotspot interfaces at hardware limits, without internet round-trips.
* 🔒 **100% Private & Local-First**: No centralized servers. Files are sent directly from sender to receiver. Your data never touches the cloud.
* 📱 **Web Share (App-less Browser Receive)**: Share files with devices that don't have the MeshDrop client installed (e.g., phones, tablets, or other computers) by generating a local QR code and URL that opens a responsive web download page in any browser.
* 🚀 **TCP Nagle Optimization**: Features `nodelay` socket options on connections to deliver low-latency, high-throughput network packets.
* 💻 **Cross-Platform Ecosystem**: Supports native builds for Windows (`.msi`), macOS (`.dmg`), and Linux (`.deb`) alongside cross-device browser transfers.
* 🛠️ **System Adapter Diagnostics**: Built-in settings shortlinks that launch native configuration panels on Windows, macOS, and Linux to quickly toggle Wi-Fi, Bluetooth, or Hotspots when needed.

---

## 🏗️ Architecture

MeshDrop is built using a dual-engine architecture:

1. **Frontend (React + TypeScript + CSS)**: A high-performance, responsive UI featuring smooth glassmorphism, dynamic scanning radar animations, and clean detail layouts.
2. **Backend (Rust + Tauri)**: Handles OS-level operations, offline IP route resolution (prioritizing physical adapters like `192.168.x.x` over virtual interfaces or Cloudflare WARP/VPN adapters), and hosts the local HTTP Web Share server using Rust's standard library `TcpListener`.

---

## 🛠️ Development & Build Setup

### Prerequisites
* **Node.js** (v18+)
* **Rust & Cargo** (Latest stable)
* System-specific Tauri dependencies (see the [Tauri Setup Guide](https://tauri.app/v1/guides/getting-started/prerequisites)).

### Local Development
To launch the application in development mode with hot-reloading:

1. Install frontend dependencies:
   ```bash
   npm install
   ```
2. Start the Tauri development environment:
   ```bash
   npm run tauri dev
   ```

### Packaging & Compiling (Release Build)
To compile a optimized standalone installer executable:

```bash
npm run tauri build
```
* **Windows**: Generates a `.msi` installer in `src-tauri/target/release/bundle/msi/`.
* **Linux**: Generates a `.deb` package in `src-tauri/target/release/bundle/deb/`.
* **macOS**: Generates a `.dmg` application bundle in `src-tauri/target/release/bundle/dmg/`.

---

## 🌍 Web Share & Native Installer Setup
The continuous HTTP web server runs locally on port `7880`. 

* **To serve real installers to other devices on your LAN**:
  Create an `installers/` folder next to your compiled MeshDrop executable and place the production build files in it:
  - `meshdrop.msi` (Windows)
  - `meshdrop.deb` (Linux)
  - `meshdrop.dmg` (macOS)
  - `meshdrop.apk` (Android)
  
  When visitors click the "Download Native Client" button from the browser landing page, the server will stream the corresponding installation package. If missing, the server serves a mock fallback file to verify UI routing.

---

## 💡 Troubleshooting & Performance Tips

### Slow Speeds (Throttled Transfers)
If your file transfer rate is slow (e.g., capped under 2 Mbps):
* **Disable VPNs / Cloudflare WARP**: Virtual Private Networks intercept local IP addresses and route packets through remote internet servers. Turn off WARP or VPNs on both the sender and receiver devices to allow direct local routing over your home WiFi.

### Connection Failures
* **Same Network (LAN)**: Ensure both devices are connected to the exact same WiFi router or mobile hotspot.
* **Firewall Access**: Ensure your firewall permits incoming connections on port `7878` (MeshDrop direct packets) and port `7880` (Web Share server).
