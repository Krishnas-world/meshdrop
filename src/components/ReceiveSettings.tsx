type ReceiveSettingsProps = {
  receiveFolder: string;
  onReceiveFolderChange: (folder: string) => void;
};

export function ReceiveSettings({ receiveFolder, onReceiveFolderChange }: ReceiveSettingsProps) {
  return (
    <section className="panel settings-panel">
      <div>
        <p className="eyebrow">Storage</p>
        <h2>Receive folder</h2>
      </div>

      <label className="field">
        <span>Save incoming files to</span>
        <input
          type="text"
          value={receiveFolder}
          onChange={(event) => onReceiveFolderChange(event.target.value)}
        />
      </label>

      <div className="settings-note">
        <strong>Current V1 behavior</strong>
        <p>The folder is created on accept, then incoming file bytes are saved there.</p>
      </div>
    </section>
  );
}
