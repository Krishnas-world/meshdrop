import type { NearbyDevice, TransportOption } from "../types";

type ConnectionPanelProps = {
  ip: string;
  devices: NearbyDevice[];
  transports: TransportOption[];
  serverStatus: "idle" | "online" | "error";
  onIpChange: (ip: string) => void;
  onStartServer: () => void;
  onStartDiscovery: () => void;
  onStartDirectConnect: () => void;
  onOpenTransportAction: (action: string) => void;
  onSelectDevice: (device: NearbyDevice) => void;
  onSendMessage: () => void;
};

export function ConnectionPanel({
  ip,
  devices,
  transports,
  serverStatus,
  onIpChange,
  onStartServer,
  onStartDiscovery,
  onStartDirectConnect,
  onOpenTransportAction,
  onSelectDevice,
  onSendMessage,
}: ConnectionPanelProps) {
  const realDevices = devices.filter(
    (device) => device.id !== "manual-peer" && device.id !== "discovery-placeholder",
  );
  const primaryRoutes = transports.filter((transport) =>
    ["bluetooth-proximity", "auto-hotspot", "wifi-direct"].includes(transport.id),
  );
  const fallbackRoutes = transports.filter((transport) =>
    ["qr-pairing", "lan-fallback", "chunked-tcp"].includes(transport.id),
  );

  return (
    <section className="panel connection-panel">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">Step 1</p>
          <h2>Direct Connect</h2>
        </div>
        <button className="ghost-button" onClick={onStartServer}>
          {serverStatus === "online" ? "Listening" : "Start receiver"}
        </button>
      </div>

      <div className="scan-stage">
        <div>
          <p className="eyebrow">Nearby</p>
          <h3>Find devices without sharing WiFi</h3>
          <p>
            MeshDrop will try Bluetooth/QR for discovery, then hotspot or WiFi Direct for the fast link.
          </p>
        </div>
        <button className="primary-button" onClick={onStartDirectConnect}>
          Start direct scan
        </button>
      </div>

      <div className="transport-grid">
        {primaryRoutes.map((transport) => (
          <article className="transport-route" key={transport.id}>
            <div>
              <strong>{transport.name}</strong>
              <small>{transport.detail}</small>
            </div>
            <div className="transport-route-actions">
              <em>{formatStatus(transport.status)}</em>
              {transport.action && (
                <button
                  className="mini-button"
                  onClick={() => onOpenTransportAction(transport.action ?? "")}
                >
                  Open
                </button>
              )}
            </div>
          </article>
        ))}
      </div>

      <div className="section-heading-row">
        <div>
          <p className="eyebrow">Devices</p>
          <h3>Discovered devices</h3>
        </div>
        <button className="ghost-button" onClick={onStartDiscovery}>
          LAN fallback
        </button>
      </div>

      <div className="device-list">
        {realDevices.length === 0 ? (
          <div className="empty-device-state">
            <strong>No devices found yet</strong>
            <span>Start direct scan, turn on Bluetooth/WiFi, or use the fallback IP below.</span>
          </div>
        ) : (
          realDevices.map((device) => (
            <button
              className={`device-card ${ip === device.address ? "device-card-selected" : ""}`}
              type="button"
              key={device.id}
              onClick={() => onSelectDevice(device)}
            >
              <span className="device-avatar">{device.name.slice(0, 1)}</span>
              <span>
                <strong>{device.name}</strong>
                <small>{device.address}</small>
              </span>
              <em>{device.status}</em>
            </button>
          ))
        )}
      </div>

      <details className="fallback-panel">
        <summary>Advanced fallback</summary>
        <label className="field">
          <span>Receiver IP</span>
          <input
            type="text"
            placeholder="192.168.x.x"
            value={ip}
            onChange={(event) => onIpChange(event.target.value)}
          />
        </label>

        <div className="transport-grid compact">
          {fallbackRoutes.map((transport) => (
            <article className="transport-route" key={transport.id}>
              <div>
                <strong>{transport.name}</strong>
                <small>{transport.detail}</small>
              </div>
              <em>{formatStatus(transport.status)}</em>
            </article>
          ))}
        </div>

        <button className="ghost-button" onClick={onSendMessage}>
          Send test message
        </button>
      </details>
    </section>
  );
}

function formatStatus(status: string) {
  const labels: Record<string, string> = {
    "needs-permission": "Needs permission",
    "ready-design": "Designed",
    available: "Available",
    scanning: "Scanning",
    ready: "Ready",
    next: "Next",
    planned: "Planned",
  };

  return labels[status] ?? status;
}
