import { useRef } from "react";
import { formatBytes } from "../utils/format";

type SendPanelProps = {
  selectedFile: File | null;
  canSend: boolean;
  onFileChange: (file: File | null) => void;
  onSendOffer: () => void;
};

export function SendPanel({ selectedFile, canSend, onFileChange, onSendOffer }: SendPanelProps) {
  const inputRef = useRef<HTMLInputElement | null>(null);

  return (
    <section className="panel send-panel">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">Step 2</p>
          <h2>Pick a file</h2>
        </div>
        <button className="ghost-button" onClick={() => inputRef.current?.click()}>
          Browse
        </button>
      </div>

      <button className="drop-zone" type="button" onClick={() => inputRef.current?.click()}>
        <span className="drop-icon">+</span>
        <strong>{selectedFile ? selectedFile.name : "Choose a file to drop"}</strong>
        <small>
          {selectedFile
            ? `${formatBytes(selectedFile.size)} ready to offer`
            : "V1 sends one file over TCP after receiver approval"}
        </small>
      </button>

      <input
        ref={inputRef}
        className="visually-hidden"
        type="file"
        onChange={(event) => onFileChange(event.target.files?.[0] ?? null)}
      />

      <button className="primary-button wide-button" disabled={!canSend} onClick={onSendOffer}>
        Send offer
      </button>
    </section>
  );
}
