import { open } from "@tauri-apps/plugin-dialog";
import type { IncomingFile } from "../types";
import { formatBytes } from "../utils/format";

type IncomingFileDialogProps = {
  file: IncomingFile;
  receiveFolder: string;
  onAccept: (receiveLocation?: string) => void;
  onReject: () => void;
};

export function IncomingFileDialog({
  file,
  receiveFolder,
  onAccept,
  onReject,
}: IncomingFileDialogProps) {
  async function chooseOtherLocation() {
    const selected = await open({
      directory: true,
      multiple: false,
      title: "Choose where MeshDrop should save this file",
    });

    if (typeof selected === "string") {
      onAccept(selected);
    }
  }

  return (
    <div className="dialog-backdrop" role="presentation">
      <section className="incoming-dialog" role="dialog" aria-modal="true">
        <p className="eyebrow">Incoming file</p>
        <h2>{file.name}</h2>
        <p className="dialog-size">{formatBytes(file.size)}</p>
        <div className="dialog-folder">
          <span>Default MeshDrop folder</span>
          <strong>{receiveFolder}</strong>
        </div>

        <div className="dialog-actions">
          <button className="ghost-button" onClick={onReject}>
            Reject
          </button>
          <button className="ghost-button" onClick={chooseOtherLocation}>
            Choose other
          </button>
          <button className="primary-button" onClick={() => onAccept()}>
            Use default
          </button>
        </div>
      </section>
    </div>
  );
}
