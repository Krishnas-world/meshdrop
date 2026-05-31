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
  webShareActive: boolean;
  webShareUrl: string;
  onStartWebShare: () => void;
  onStopWebShare: () => void;
};

export function SendPanel({
  selectedFile,
  selectedFilePath,
  needsFilePath,
  canSend,
  onFileChange,
  onFilePathChange,
  onSendOffer,
  webShareActive,
  webShareUrl,
  onStartWebShare,
  onStopWebShare,
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

      <div className="send-actions-container">
        <button className="primary-button send-offer-btn" disabled={!canSend} onClick={onSendOffer}>
          Send offer
        </button>

        {selectedFilePath && (
          <div className="web-share-controls">
            {webShareActive ? (
              <div className="web-share-card">
                <div className="web-share-header-row">
                  <span className="pulse-dot"></span>
                  <strong>Web Share Active</strong>
                </div>
                <p className="web-share-url-text">
                  Open this URL in your phone or other device browser:
                </p>
                <code className="web-share-link">{webShareUrl}</code>
                <div className="qr-container">
                  <img
                    src={`https://api.qrserver.com/v1/create-qr-code/?size=140x140&color=17201b&bgcolor=f7f9f5&data=${encodeURIComponent(webShareUrl)}`}
                    alt="QR Code to scan link"
                    className="web-share-qr"
                  />
                </div>
                <button className="ghost-button stop-share-button" onClick={onStopWebShare}>
                  Stop Web Share
                </button>
              </div>
            ) : (
              <button
                className="ghost-button web-share-button"
                type="button"
                onClick={onStartWebShare}
              >
                🌐 Share to phone/other devices via Browser
              </button>
            )}
          </div>
        )}
      </div>
    </section>
  );
}
