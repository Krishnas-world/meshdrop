import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { useRef } from "react";
import { formatBytes } from "../utils/format";

type SendPanelProps = {
  selectedFile: File | null;
  selectedFilePath: string;
  needsFilePath: boolean;
  canSend: boolean;
  onFileChange: (file: File | null) => void;
  onFilePathChange: (path: string) => void;
  onSendOffer: () => void;
};

export function SendPanel({
  selectedFile,
  selectedFilePath,
  needsFilePath,
  canSend,
  onFileChange,
  onFilePathChange,
  onSendOffer,
}: SendPanelProps) {
  const inputRef = useRef<HTMLInputElement | null>(null);

  async function chooseFile() {
    const selected = await open({
      directory: false,
      multiple: false,
      title: "Choose file to send",
    });

    if (typeof selected !== "string") {
      return;
    }

    onFilePathChange(selected);

    const info = await invoke<string>("get_file_info", {
      path: selected,
    });
    const [name, size] = info.split("|");

    onFileChange({
      name,
      size: Number(size),
      path: selected,
    } as File & { path: string });
  }

  return (
    <section className="panel send-panel">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">Step 2</p>
          <h2>Pick a file</h2>
        </div>
        <button className="ghost-button" onClick={chooseFile}>
          Choose file
        </button>
      </div>

      <button className="drop-zone" type="button" onClick={chooseFile}>
        <span className="drop-icon">+</span>
        <strong>{selectedFile ? selectedFile.name : "Choose a file to drop"}</strong>
        <small>
          {selectedFile && selectedFile.size > 0
            ? `${formatBytes(selectedFile.size)} ready to offer`
            : selectedFilePath
              ? "Ready to stream from disk after receiver approval"
              : "V1 sends one file over TCP after receiver approval"}
        </small>
      </button>

      <input
        ref={inputRef}
        className="visually-hidden"
        type="file"
        onChange={(event) => onFileChange(event.target.files?.[0] ?? null)}
      />

      {selectedFilePath && (
        <p className="selected-path">
          {selectedFilePath}
        </p>
      )}

      {needsFilePath && !selectedFilePath && (
        <p className="path-warning">
          Choose the file with the native picker so MeshDrop can stream it from Rust.
        </p>
      )}

      <button className="primary-button wide-button" disabled={!canSend} onClick={onSendOffer}>
        Send offer
      </button>
    </section>
  );
}
