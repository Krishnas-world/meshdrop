import type { IncomingFile } from "../types";
import { formatBytes } from "../utils/format";

type IncomingFileDialogProps = {
  file: IncomingFile;
  receiveFolder: string;
  onAccept: () => void;
  onReject: () => void;
};

export function IncomingFileDialog({
  file,
  receiveFolder,
  onAccept,
  onReject,
}: IncomingFileDialogProps) {
  return (
    <div className="dialog-backdrop" role="presentation">
      <section className="incoming-dialog" role="dialog" aria-modal="true">
        <p className="eyebrow">Incoming file</p>
        <h2>{file.name}</h2>
        <p className="dialog-size">{formatBytes(file.size)}</p>
        <p className="dialog-folder">{receiveFolder}</p>

        <div className="dialog-actions">
          <button className="ghost-button" onClick={onReject}>
            Reject
          </button>
          <button className="primary-button" onClick={onAccept}>
            Accept and save
          </button>
        </div>
      </section>
    </div>
  );
}
