import { open } from "@tauri-apps/plugin-dialog";

type ReceiveSettingsProps = {
  receiveFolder: string;
  onChooseReceiveLocation: (location: string) => void;
};

export function ReceiveSettings({
  receiveFolder,
  onChooseReceiveLocation,
}: ReceiveSettingsProps) {
  async function chooseLocation() {
    const selected = await open({
      directory: true,
      multiple: false,
      title: "Choose where MeshDrop should save received files",
    });

    if (typeof selected === "string") {
      onChooseReceiveLocation(selected);
    }
  }

  return (
    <section className="panel settings-panel">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">Storage</p>
          <h2>Receive location</h2>
        </div>
        <button className="ghost-button" onClick={chooseLocation}>
          Choose location
        </button>
      </div>

      <div className="folder-preview">
        <span>MeshDrop folder</span>
        <strong>{receiveFolder}</strong>
      </div>

      <div className="settings-note">
        <strong>Current V1 behavior</strong>
        <p>Pick any location. MeshDrop creates and uses its own MeshDrop folder inside it.</p>
      </div>
    </section>
  );
}
