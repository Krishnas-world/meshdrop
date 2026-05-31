import type { NearbyDevice } from "../types";

type ConnectionPanelProps = {
  ip: string;
  devices: NearbyDevice[];
  serverStatus: "idle" | "online" | "error";
  onIpChange: (ip: string) => void;
  onStartServer: () => void;
  onSendMessage: () => void;
};

export function ConnectionPanel({
  ip,
  devices,
  serverStatus,
  onIpChange,
  onStartServer,
  onSendMessage,
}: ConnectionPanelProps) {
  return (
    <section className="panel connection-panel">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">Step 1</p>
          <h2>Connect</h2>
        </div>
        <button className="primary-button" onClick={onStartServer}>
          {serverStatus === "online" ? "Listening" : "Start receiver"}
        </button>
      </div>

      <label className="field">
        <span>Receiver IP</span>
        <input
          type="text"
          placeholder="192.168.x.x"
          value={ip}
          onChange={(event) => onIpChange(event.target.value)}
        />
      </label>

      <div className="device-list">
        {devices.map((device) => (
          <button className="device-card" type="button" key={device.name}>
            <span className="device-avatar">{device.name.slice(0, 1)}</span>
            <span>
              <strong>{device.name}</strong>
              <small>{device.address}</small>
            </span>
            <em>{device.status}</em>
          </button>
        ))}
      </div>

      <button className="ghost-button" onClick={onSendMessage}>
        Send test message
      </button>
    </section>
  );
}
