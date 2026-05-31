import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { openUrl } from "@tauri-apps/plugin-opener";
import { useEffect, useMemo, useRef, useState } from "react";
import { ConnectionPanel } from "./components/ConnectionPanel";
import { IncomingFileDialog } from "./components/IncomingFileDialog";
import { ProductPlan } from "./components/ProductPlan";
import { ReceiveSettings } from "./components/ReceiveSettings";
import { SendPanel } from "./components/SendPanel";
import { TransferActivity } from "./components/TransferActivity";
import type { ActivityItem, IncomingFile, NearbyDevice, TransportOption } from "./types";

function parseIncomingFile(payload: string): IncomingFile | null {
  const [id, name, size] = payload.split("|");

  if (!id || !name || !size) {
    return null;
  }

  return {
    id,
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
  const [selectedFilePath, setSelectedFilePath] = useState("");
  const [outgoingTransferId, setOutgoingTransferId] = useState("");
  const [receiveFolder, setReceiveFolder] = useState("");
  const [serverStatus, setServerStatus] = useState<"idle" | "online" | "error">("idle");
  const [transferStatus, setTransferStatus] = useState("Ready for nearby transfers");
  const [progress, setProgress] = useState(0);
  const [activity, setActivity] = useState<ActivityItem[]>([]);
  const [discoveredDevices, setDiscoveredDevices] = useState<NearbyDevice[]>([]);
  const [transports, setTransports] = useState<TransportOption[]>([]);
  const selectedFileRef = useRef<File | null>(null);
  const selectedFilePathRef = useRef("");
  const outgoingTransferIdRef = useRef("");
  const incomingTransferIdRef = useRef("");
  const ipRef = useRef("");

  const needsFilePath = Boolean(selectedFile && selectedFile.size > 25 * 1024 * 1024);
  const canSend = Boolean(ip.trim() && selectedFile && (!needsFilePath || selectedFilePath.trim()));

  const nearbyDevices = useMemo(
    () => [
      ...discoveredDevices,
      {
        id: "manual-peer",
        name: "Manual direct address",
        address: ip.trim() || "Enter IP",
        status: ip.trim() ? "Ready" : "Needs address",
      },
      {
        id: "discovery-placeholder",
        name: "Fallback discovery",
        address: "UDP broadcast",
        status: discoveredDevices.length > 0 ? "Listening" : "Find devices",
      },
    ],
    [discoveredDevices, ip],
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

  function upsertTransferActivity(
    transferId: string,
    item: Omit<ActivityItem, "id" | "time" | "transferId">,
  ) {
    setActivity((current) => {
      const now = new Date().toLocaleTimeString([], {
        hour: "2-digit",
        minute: "2-digit",
      });
      const existing = current.find((activityItem) => activityItem.transferId === transferId);

      if (!existing) {
        return [
          {
            ...item,
            id: transferId,
            transferId,
            time: now,
          },
          ...current,
        ];
      }

      return current.map((activityItem) =>
        activityItem.transferId === transferId
          ? {
              ...activityItem,
              ...item,
              time: now,
            }
          : activityItem,
      );
    });
  }

  useEffect(() => {
    selectedFileRef.current = selectedFile;
  }, [selectedFile]);

  useEffect(() => {
    selectedFilePathRef.current = selectedFilePath;
  }, [selectedFilePath]);

  useEffect(() => {
    outgoingTransferIdRef.current = outgoingTransferId;
  }, [outgoingTransferId]);

  useEffect(() => {
    ipRef.current = ip;
  }, [ip]);

  useEffect(() => {
    invoke<string>("get_receive_folder")
      .then(setReceiveFolder)
      .catch((error) => console.error("Failed to load receive folder:", error));
    invoke<TransportOption[]>("get_transport_plan")
      .then(setTransports)
      .catch((error) => console.error("Failed to load transport plan:", error));

    const unlistenMessage = listen<string>("message-received", (event) => {
      setReceivedMessage(event.payload);
      addActivity({
        title: "Message received",
        detail: event.payload,
        status: "done",
        direction: "message",
      });
    });

    const unlistenDeviceDiscovered = listen<string>("device-discovered", (event) => {
      const [id, name, address, port] = event.payload.split("|");

      if (!id || !name || !address) {
        return;
      }

      setDiscoveredDevices((current) => {
        const device = {
          id,
          name,
          address,
          port: Number(port),
          status: "Online",
          lastSeen: Date.now(),
        };

        if (!current.some((item) => item.id === id)) {
          return [device, ...current];
        }

        return current.map((item) => (item.id === id ? device : item));
      });
    });

    const unlistenTransportStatus = listen<string>("transport-status", (event) => {
      const route = JSON.parse(event.payload) as TransportOption;

      setTransports((current) =>
        current.map((item) => (item.id === route.id ? route : item)),
      );
    });

    const unlistenFileOffer = listen<string>("incoming-file-offer", (event) => {
      const file = parseIncomingFile(event.payload);
      setIncomingFile(file);
      setTransferStatus("Incoming file offer");
      setProgress(10);

      if (file) {
        incomingTransferIdRef.current = file.id;
        upsertTransferActivity(file.id, {
          title: `Incoming ${file.name}`,
          detail: "Waiting for your response",
          status: "waiting",
          progress: 10,
          direction: "receive",
        });
      }
    });

    const unlistenAccepted = listen<string>("file-accepted", async (event) => {
      const acceptedTransferId = event.payload;
      const file = selectedFileRef.current;
      const filePath = selectedFilePathRef.current.trim();
      const transferId = outgoingTransferIdRef.current;
      const targetIp = ipRef.current.trim();

      if (acceptedTransferId !== transferId) {
        return;
      }

      if (!file || !targetIp) {
        setTransferStatus("Receiver accepted, but no selected file is ready.");
        setProgress(0);
        return;
      }

      try {
        setTransferStatus(`Sending ${file.name}`);
        setProgress(20);

        if (filePath) {
          await invoke("send_file_from_path", {
            ip: targetIp,
            transferId,
            path: filePath,
          });
        } else if (file.size <= 25 * 1024 * 1024) {
          const bytes = Array.from(new Uint8Array(await file.arrayBuffer()));

          await invoke("send_file_data", {
            ip: targetIp,
            transferId,
            filename: file.name,
            bytes,
          });
        } else {
          setTransferStatus("Large files need the local file path field before sending.");
          setProgress(0);
          return;
        }

        setProgress(100);
        setTransferStatus(`Sent ${file.name}`);
        setOutgoingTransferId("");
        outgoingTransferIdRef.current = "";
        upsertTransferActivity(transferId, {
          title: `Sent ${file.name}`,
          detail: `${file.size.toLocaleString()} bytes to ${targetIp}`,
          status: "done",
          progress: 100,
          direction: "send",
        });
      } catch (error) {
        console.error("Failed to send file data:", error);
        setTransferStatus("Failed to send file data");
        setProgress(0);
        upsertTransferActivity(transferId, {
          title: "Send failed",
          detail: file.name,
          status: "failed",
          progress: 0,
          direction: "send",
        });
      }
    });

    const unlistenRejected = listen<string>("file-rejected", (event) => {
      if (event.payload !== outgoingTransferIdRef.current) {
        return;
      }

      setTransferStatus("Receiver rejected the file");
      setOutgoingTransferId("");
      outgoingTransferIdRef.current = "";
      setProgress(0);
      upsertTransferActivity(event.payload, {
        title: "Transfer rejected",
        detail: selectedFileRef.current?.name ?? "File offer",
        status: "failed",
        progress: 0,
        direction: "send",
      });
    });

    const unlistenFileReceived = listen<string>("file-received", (event) => {
      const [transferId, path] = event.payload.split("|");
      if (incomingTransferIdRef.current && transferId !== incomingTransferIdRef.current) {
        return;
      }

      setProgress(100);
      setTransferStatus(`Saved to ${path}`);
      upsertTransferActivity(transferId, {
        title: "File received",
        detail: path,
        status: "done",
        progress: 100,
        direction: "receive",
      });
      incomingTransferIdRef.current = "";
    });

    const unlistenSendProgress = listen<string>("file-send-progress", (event) => {
      const [transferId, name, sent, total, percent] = event.payload.split("|");
      if (transferId !== outgoingTransferIdRef.current) {
        return;
      }

      setProgress(Number(percent));
      setTransferStatus(`Sending ${name}: ${Number(sent).toLocaleString()} / ${Number(total).toLocaleString()} bytes`);
      upsertTransferActivity(transferId, {
        title: `Sending ${name}`,
        detail: `${Number(sent).toLocaleString()} / ${Number(total).toLocaleString()} bytes`,
        status: "active",
        progress: Number(percent),
        direction: "send",
      });
    });

    const unlistenReceiveProgress = listen<string>("file-receive-progress", (event) => {
      const [transferId, name, received, total, percent] = event.payload.split("|");
      if (incomingTransferIdRef.current && transferId !== incomingTransferIdRef.current) {
        return;
      }

      setProgress(Number(percent));
      setTransferStatus(`Receiving ${name}: ${Number(received).toLocaleString()} / ${Number(total).toLocaleString()} bytes`);
      upsertTransferActivity(transferId, {
        title: `Receiving ${name}`,
        detail: `${Number(received).toLocaleString()} / ${Number(total).toLocaleString()} bytes`,
        status: "active",
        progress: Number(percent),
        direction: "receive",
      });
    });

    return () => {
      unlistenMessage.then((fn) => fn());
      unlistenDeviceDiscovered.then((fn) => fn());
      unlistenTransportStatus.then((fn) => fn());
      unlistenFileOffer.then((fn) => fn());
      unlistenAccepted.then((fn) => fn());
      unlistenRejected.then((fn) => fn());
      unlistenFileReceived.then((fn) => fn());
      unlistenSendProgress.then((fn) => fn());
      unlistenReceiveProgress.then((fn) => fn());
    };
  }, []);

  async function startServer() {
    try {
      await invoke("start_server");
      await startDiscovery();
      setServerStatus("online");
      setTransferStatus("Receiver listening on port 7878");
    } catch (error) {
      console.error("Failed to start server:", error);
      setServerStatus("error");
      setTransferStatus("Could not start receiver");
    }
  }

  async function startDiscovery() {
    try {
      const deviceName = await invoke<string>("start_discovery");
      setTransferStatus(`Finding nearby devices as ${deviceName}`);
    } catch (error) {
      console.error("Failed to start discovery:", error);
      setTransferStatus("Could not start fallback discovery");
    }
  }

  async function startDirectConnect() {
    try {
      const plan = await invoke<TransportOption[]>("start_direct_connect");
      setTransports(plan);
      setTransferStatus("Direct scan started. Turn on Bluetooth/WiFi or use LAN fallback while adapters come online.");
    } catch (error) {
      console.error("Failed to start direct connect:", error);
      setTransferStatus(`Could not start direct scan: ${String(error)}`);
    }
  }

  async function openTransportAction(action: string) {
    const urls: Record<string, string> = {
      "bluetooth-settings": "ms-settings:bluetooth",
      "wifi-settings": "ms-settings:network-wifi",
      "hotspot-settings": "ms-settings:network-mobilehotspot",
    };

    if (action === "qr-pairing") {
      setTransferStatus("QR pairing screen is next in v0.3.");
      return;
    }

    try {
      const url = urls[action];

      if (!url) {
        setTransferStatus("No system action is wired for this transport yet.");
        return;
      }

      await openUrl(url);
      setTransferStatus("Opened system settings. Turn on the required adapter, then start direct scan again.");
    } catch (error) {
      console.error("Failed to open transport settings:", error);
      setTransferStatus("Could not open system settings for this adapter.");
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
        direction: "message",
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

    if (needsFilePath && !selectedFilePath.trim()) {
      setTransferStatus("Paste the local file path before sending this large file.");
      return;
    }

    try {
      const transferId = await invoke<string>("send_file_offer", {
        ip,
        filename: selectedFile.name,
        filesize: selectedFile.size,
      });

      setOutgoingTransferId(transferId);
      outgoingTransferIdRef.current = transferId;
      setProgress(15);
      setTransferStatus(`Offer sent for ${selectedFile.name}`);
      upsertTransferActivity(transferId, {
        title: `Offer sent`,
        detail: `${selectedFile.name} to ${ip}`,
        status: "waiting",
        progress: 15,
        direction: "send",
      });
    } catch (error) {
      console.error("Failed to send file offer:", error);
      setTransferStatus("Failed to send file offer");
    }
  }

  async function acceptFile(receiveLocation?: string) {
    if (!incomingFile) {
      return;
    }

    try {
      const savedFolder = receiveLocation
        ? await invoke<string>("set_receive_location", {
            path: receiveLocation,
          })
        : await invoke<string>("set_receive_folder", {
            path: receiveFolder,
          });

      setReceiveFolder(savedFolder);
      incomingTransferIdRef.current = incomingFile.id;

      await invoke("send_file_accept", {
        ip,
        transferId: incomingFile.id,
      });

      setIncomingFile(null);
      setProgress(45);
      setTransferStatus("Accepted. Waiting for file data...");
      upsertTransferActivity(incomingFile.id, {
        title: `Receiving ${incomingFile.name}`,
        detail: "Accepted. Waiting for sender...",
        status: "active",
        progress: 45,
        direction: "receive",
      });
    } catch (error) {
      console.error(error);
      setTransferStatus("Could not accept file. Check the receive folder path.");
    }
  }

  async function rejectFile() {
    if (!incomingFile) {
      return;
    }

    try {
      await invoke("send_file_reject", {
        ip,
        transferId: incomingFile.id,
      });

      setIncomingFile(null);
      incomingTransferIdRef.current = "";
      setProgress(0);
      setTransferStatus("File rejected");
      upsertTransferActivity(incomingFile.id, {
        title: `Rejected ${incomingFile.name}`,
        detail: "Incoming transfer rejected",
        status: "failed",
        progress: 0,
        direction: "receive",
      });
    } catch (error) {
      console.error(error);
      setTransferStatus("Could not reject file");
    }
  }

  async function chooseReceiveLocation(path: string) {
    try {
      const folder = await invoke<string>("set_receive_location", {
        path,
      });

      setReceiveFolder(folder);
      setTransferStatus(`Receive folder set to ${folder}`);
    } catch (error) {
      console.error(error);
      setTransferStatus("Could not set receive location");
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
            transports={transports}
            serverStatus={serverStatus}
            onIpChange={setIp}
            onStartServer={startServer}
            onStartDiscovery={startDiscovery}
            onStartDirectConnect={startDirectConnect}
            onOpenTransportAction={openTransportAction}
            onSelectDevice={(device) => {
              if (device.id === "discovery-placeholder") {
                startDiscovery();
                return;
              }

              if (device.address !== "Enter IP") {
                setIp(device.address);
                setTransferStatus(`Selected ${device.name} at ${device.address}`);
              }
            }}
            onSendMessage={sendMessage}
          />

          <SendPanel
            selectedFile={selectedFile}
            selectedFilePath={selectedFilePath}
            needsFilePath={needsFilePath}
            canSend={canSend}
            onFileChange={(file) => {
              setSelectedFile(file);
              setSelectedFilePath((file as (File & { path?: string }) | null)?.path ?? "");
            }}
            onFilePathChange={setSelectedFilePath}
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
            onChooseReceiveLocation={chooseReceiveLocation}
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
