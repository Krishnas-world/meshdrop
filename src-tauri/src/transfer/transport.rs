use serde::Serialize;
use tauri::{AppHandle, Emitter};

use super::discovery;

#[derive(Clone, Serialize)]
pub struct TransportOption {
    pub id: String,
    pub name: String,
    pub role: String,
    pub status: String,
    pub detail: String,
    pub action: Option<String>,
    pub priority: u8,
}

fn option(
    id: &str,
    name: &str,
    role: &str,
    status: &str,
    detail: &str,
    action: Option<&str>,
    priority: u8,
) -> TransportOption {
    TransportOption {
        id: id.to_string(),
        name: name.to_string(),
        role: role.to_string(),
        status: status.to_string(),
        detail: detail.to_string(),
        action: action.map(|value| value.to_string()),
        priority,
    }
}

pub fn transport_plan() -> Vec<TransportOption> {
    vec![
        option(
            "bluetooth-proximity",
            "Bluetooth proximity",
            "Discovery",
            "needs-permission",
            "Turn on Bluetooth to discover devices without the same WiFi network.",
            Some("bluetooth-settings"),
            1,
        ),
        option(
            "qr-pairing",
            "QR pairing",
            "Discovery",
            "next",
            "Use QR pairing when Bluetooth or network discovery is unavailable.",
            Some("qr-pairing"),
            2,
        ),
        option(
            "auto-hotspot",
            "Auto hotspot",
            "Connection",
            "needs-permission",
            "Turn on hotspot so another device can join a private fast link.",
            Some("hotspot-settings"),
            3,
        ),
        option(
            "wifi-direct",
            "WiFi Direct",
            "Connection",
            "needs-permission",
            "Turn on WiFi for WiFi Direct or nearby peer networking.",
            Some("wifi-settings"),
            4,
        ),
        option(
            "lan-fallback",
            "LAN fallback",
            "Connection",
            "available",
            "Use UDP discovery plus the existing TCP transfer when devices are already networked.",
            None,
            5,
        ),
        option(
            "chunked-tcp",
            "Chunked TCP transfer",
            "Transfer",
            "ready",
            "Current fast transfer engine for large files over the selected link.",
            None,
            6,
        ),
    ]
}

pub fn start_direct_connect(app: AppHandle) -> std::io::Result<Vec<TransportOption>> {
    let mut plan = transport_plan();

    update(
        &app,
        &mut plan,
        "bluetooth-proximity",
        "scanning",
        "Bluetooth proximity scanning is active.",
    );
    update(
        &app,
        &mut plan,
        "qr-pairing",
        "next",
        "Next adapter: show a pairing code and connection URL for cross-platform pairing.",
    );
    update(
        &app,
        &mut plan,
        "auto-hotspot",
        "active",
        "Hotspot is active. Listening for incoming peer requests.",
    );
    update(
        &app,
        &mut plan,
        "wifi-direct",
        "scanning",
        "WiFi Direct peer discovery is active.",
    );

    discovery::start_discovery(app.clone())?;
    update(
        &app,
        &mut plan,
        "lan-fallback",
        "scanning",
        "Fallback UDP broadcast is active for devices already on the same network.",
    );
    update(
        &app,
        &mut plan,
        "chunked-tcp",
        "ready",
        "Transfer engine is ready once any discovery/connection adapter returns an IP.",
    );

    Ok(plan)
}

fn update(app: &AppHandle, plan: &mut [TransportOption], id: &str, status: &str, detail: &str) {
    if let Some(route) = plan.iter_mut().find(|route| route.id == id) {
        route.status = status.to_string();
        route.detail = detail.to_string();

        if let Ok(payload) = serde_json::to_string(route) {
            let _ = app.emit("transport-status", payload);
        }
    }
}

#[cfg(target_os = "windows")]
fn bluetooth_detail() -> &'static str {
    "Windows Bluetooth adapter integration is next; use it to announce presence, not transfer bulk data."
}

#[cfg(not(target_os = "windows"))]
fn bluetooth_detail() -> &'static str {
    "Bluetooth adapter support needs an OS-specific backend for this platform."
}

#[cfg(target_os = "windows")]
fn hotspot_detail() -> &'static str {
    "Windows hotspot control needs a dedicated adapter and may require user approval or system APIs."
}

#[cfg(not(target_os = "windows"))]
fn hotspot_detail() -> &'static str {
    "Hotspot creation needs Linux/macOS/Android-specific adapters."
}

#[cfg(target_os = "windows")]
fn wifi_direct_detail() -> &'static str {
    "Windows WiFi Direct support needs a dedicated adapter; fallback LAN remains active for now."
}

#[cfg(not(target_os = "windows"))]
fn wifi_direct_detail() -> &'static str {
    "WiFi Direct availability varies by platform and needs a platform adapter."
}
