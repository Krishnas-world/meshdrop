const phases = [
  {
    title: "v0.2 Ready",
    items: [
      "Manual LAN IP",
      "Native file picker",
      "Accept or reject",
      "Choose save location",
      "Chunked large-file transfer",
      "Transfer IDs",
      "Progress events",
    ],
  },
  {
    title: "v0.3 Direct Connect",
    items: [
      "Bluetooth proximity discovery",
      "Auto hotspot mode",
      "WiFi Direct path",
      "Adaptive transport selection",
      "Fallback LAN discovery",
      "QR pairing fallback",
      "Device names",
      "Per-transfer queue",
      "Transfer history",
      "Notifications",
    ],
  },
  {
    title: "v0.4 Transfer Power",
    items: [
      "Folder transfer",
      "Multiple files",
      "Directory structures",
      "Transfer resume",
      "SQLite history database",
      "Multi-device broadcast",
    ],
  },
  {
    title: "v0.5 Smart Network",
    items: [
      "TCP or QUIC transport",
      "mDNS or Zeroconf",
      "Bluetooth connection handoff",
      "Hotspot credential exchange",
      "Transport speed scoring",
      "Offline-first pairing",
    ],
  },
  {
    title: "v0.6 Ecosystem",
    items: [
      "Clipboard sync",
      "Shared Drop Zone",
      "Nearby messaging",
      "Browser receive mode",
      "Instant app-less receive",
      "Temporary collaborative rooms",
      "LAN developer mode",
    ],
  },
  {
    title: "v1.0 Production",
    items: [
      "AES-256 encryption",
      "Session keys",
      "Secure payloads",
      "Cross-platform testing",
      "Linux support",
      "macOS support",
      "Dark mode",
      "Settings page",
      "Production installers",
    ],
  },
];

export function ProductPlan() {
  return (
    <aside className="roadmap">
      <div>
        <p className="eyebrow">Product flow</p>
        <h2>Build path</h2>
      </div>

      <div className="roadmap-list">
        {phases.map((phase) => (
          <section className="roadmap-phase" key={phase.title}>
            <h3>{phase.title}</h3>
            <ul>
              {phase.items.map((item) => (
                <li key={item}>{item}</li>
              ))}
            </ul>
          </section>
        ))}
      </div>
    </aside>
  );
}
