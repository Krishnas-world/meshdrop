import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useMemo, useRef, useState } from "react";
import { ConnectionPanel } from "./components/ConnectionPanel";
import { IncomingFileDialog } from "./components/IncomingFileDialog";
import { ProductPlan } from "./components/ProductPlan";
import { ReceiveSettings } from "./components/ReceiveSettings";
import { SendPanel } from "./components/SendPanel";
import { TransferActivity } from "./components/TransferActivity";
import type { ActivityItem, IncomingFile } from "./types";

function parseIncomingFile(payload: string): IncomingFile | null {
  const [name, size] = payload.split("|");

  if (!name || !size) {
    return null;
  }

  return {
    name,
    size: Number(size),
    raw: payload,
  };
}

function App() {
  const [ip, setIp] = useState("");
  const [receivedMessage, setReceivedMessage] = useState("");
  const [incomingFile, setIncomingFile] = useState<IncomingFile | null>(null);
  const [selectedFile, setSelectedFile] = useState<File | null>(null);
  const [receiveFolder, setReceiveFolder] = useState("");
  const [serverStatus, setServerStatus] = useState<"idle" | "online" | "error">("idle");
  const [transferStatus, setTransferStatus] = useState("Ready for nearby transfers");
  const [progress, setProgress] = useState(0);
  const [activity, setActivity] = useState<ActivityItem[]>([]);
  const selectedFileRef = useRef<File | null>(null);
  const ipRef = useRef("");

  const canSend = Boolean(ip.trim() && selectedFile);

  const nearbyDevices = useMemo(
    () => [
      {
        name: "Manual LAN peer",
        address: ip.trim() || "Enter IP",
        status: ip.trim() ? "Ready" : "Needs address",
      },
      {
        name: "Device discovery",
        address: "mDNS / UDP",
        status: "Planned v0.3",
      },
    ],
    [ip],
  );

  function addActivity(item: Omit<ActivityItem, "id" | "time">) {
    setActivity((current) => [
      {
        ...item,
        id: crypto.randomUUID(),
        time: new Date().toLocaleTimeString([], {
          hour: "2-digit",
          minute: "2-digit",
        }),
      },
      ...current,
    ]);
  }

  useEffect(() => {
    selectedFileRef.current = selectedFile;
  }, [selectedFile]);

  useEffect(() => {
    ipRef.current = ip;
  }, [ip]);

  useEffect(() => {
    invoke<string>("get_receive_folder")
      .then(setReceiveFolder)
      .catch((error) => console.error("Failed to load receive folder:", error));

    const unlistenMessage = listen<string>("message-received", (event) => {
      setReceivedMessage(event.payload);
      addActivity({
        title: "Message received",
        detail: event.payload,
        status: "done",
      });
    });

    const unlistenFileOffer = listen<string>("incoming-file-offer", (event) => {
      const file = parseIncomingFile(event.payload);
      setIncomingFile(file);
      setTransferStatus("Incoming file offer");
      setProgress(10);

      if (file) {
        addActivity({
          title: `Incoming ${file.name}`,
          detail: "Waiting for your response",
          status: "waiting",
        });
      }
    });

    const unlistenAccepted = listen("file-accepted", async () => {
      const file = selectedFileRef.current;
      const targetIp = ipRef.current.trim();

      if (!file || !targetIp) {
        setTransferStatus("Receiver accepted, but no selected file is ready.");
        setProgress(0);
        return;
      }

      try {
        setTransferStatus(`Sending ${file.name}`);
        setProgress(35);

        const bytes = Array.from(new Uint8Array(await file.arrayBuffer()));
        setProgress(75);

        await invoke("send_file_data", {
          ip: targetIp,
          filename: file.name,
          bytes,
        });

        setProgress(100);
        setTransferStatus(`Sent ${file.name}`);
        addActivity({
          title: `Sent ${file.name}`,
          detail: `${file.size.toLocaleString()} bytes to ${targetIp}`,
          status: "done",
        });
      } catch (error) {
        console.error("Failed to send file data:", error);
        setTransferStatus("Failed to send file data");
        setProgress(0);
        addActivity({
          title: "Send failed",
          detail: file.name,
          status: "failed",
        });
      }
    });

    const unlistenRejected = listen("file-rejected", () => {
      setTransferStatus("Receiver rejected the file");
      setProgress(0);
      addActivity({
        title: "Transfer rejected",
        detail: selectedFileRef.current?.name ?? "File offer",
        status: "failed",
      });
    });

    const unlistenFileReceived = listen<string>("file-received", (event) => {
      setProgress(100);
      setTransferStatus(`Saved to ${event.payload}`);
      addActivity({
        title: "File received",
        detail: event.payload,
        status: "done",
      });
    });

    return () => {
      unlistenMessage.then((fn) => fn());
      unlistenFileOffer.then((fn) => fn());
      unlistenAccepted.then((fn) => fn());
      unlistenRejected.then((fn) => fn());
      unlistenFileReceived.then((fn) => fn());
    };
  }, []);

  async function startServer() {
    try {
      await invoke("start_server");
      setServerStatus("online");
      setTransferStatus("Receiver listening on port 7878");
    } catch (error) {
      console.error("Failed to start server:", error);
      setServerStatus("error");
      setTransferStatus("Could not start receiver");
    }
  }

  async function sendMessage() {
    try {
      await invoke("send_message", {
        ip,
        message: "Hello from MeshDrop",
      });

      addActivity({
        title: "Message sent",
        detail: ip,
        status: "done",
      });
    } catch (error) {
      console.error("Failed to send message:", error);
      setTransferStatus("Failed to send test message");
    }
  }

  async function sendFileOffer() {
    if (!selectedFile) {
      setTransferStatus("Select a file first");
      return;
    }

    if (!ip.trim()) {
      setTransferStatus("Enter the receiver IP first");
      return;
    }

    try {
      await invoke("send_file_offer", {
        ip,
        filename: selectedFile.name,
        filesize: selectedFile.size,
      });

      setProgress(15);
      setTransferStatus(`Offer sent for ${selectedFile.name}`);
      addActivity({
        title: `Offer sent`,
        detail: `${selectedFile.name} to ${ip}`,
        status: "waiting",
      });
    } catch (error) {
      console.error("Failed to send file offer:", error);
      setTransferStatus("Failed to send file offer");
    }
  }

  async function acceptFile() {
    if (!incomingFile) {
      return;
    }

    try {
      const savedFolder = await invoke<string>("set_receive_folder", {
        path: receiveFolder,
      });

      setReceiveFolder(savedFolder);

      await invoke("send_file_accept", {
        ip,
      });

      setIncomingFile(null);
      setProgress(45);
      setTransferStatus("Accepted. Waiting for file data...");
    } catch (error) {
      console.error(error);
      setTransferStatus("Could not accept file. Check the receive folder path.");
    }
  }

  async function rejectFile() {
    try {
      await invoke("send_file_reject", {
        ip,
      });

      setIncomingFile(null);
      setProgress(0);
      setTransferStatus("File rejected");
    } catch (error) {
      console.error(error);
      setTransferStatus("Could not reject file");
    }
  }

  return (
    <main className="app-shell">
      <section className="workspace">
        <header className="topbar">
          <div>
            <p className="eyebrow">Local-first file sharing</p>
            <h1>MeshDrop</h1>
          </div>
          <div className={`server-pill server-pill-${serverStatus}`}>
            <span />
            {serverStatus === "online" ? "Receiver online" : "Receiver idle"}
          </div>
        </header>

        <div className="dashboard-grid">
          <ConnectionPanel
            ip={ip}
            devices={nearbyDevices}
            serverStatus={serverStatus}
            onIpChange={setIp}
            onStartServer={startServer}
            onSendMessage={sendMessage}
          />

          <SendPanel
            selectedFile={selectedFile}
            canSend={canSend}
            onFileChange={setSelectedFile}
            onSendOffer={sendFileOffer}
          />

          <TransferActivity
            progress={progress}
            status={transferStatus}
            receivedMessage={receivedMessage}
            activity={activity}
          />

          <ReceiveSettings
            receiveFolder={receiveFolder}
            onReceiveFolderChange={setReceiveFolder}
          />
        </div>
      </section>

      <ProductPlan />

      {incomingFile && (
        <IncomingFileDialog
          file={incomingFile}
          receiveFolder={receiveFolder}
          onAccept={acceptFile}
          onReject={rejectFile}
        />
      )}
    </main>
  );
}

export default App;
